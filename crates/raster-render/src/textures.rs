use crate::{Gpu, Texture};
use raster_core::asset::{AssetError, AssetId, AssetStore, Handle, Project, Watcher};

/// Textures loaded from a project, each one uploaded once.
///
/// Le damier remplace toute texture manquante : un asset absent doit se voir a
/// l'ecran, pas empecher le jeu de demarrer.
pub struct Textures {
    store: AssetStore<Texture>,
    project: Project,
    placeholder: Handle<Texture>,
    missing: Vec<AssetId>,
    watcher: Watcher,
}

impl Textures {
    /// Creates the cache, uploading the placeholder straight away.
    pub fn new(gpu: &Gpu, layout: &wgpu::BindGroupLayout, project: Project) -> Self {
        let mut store = AssetStore::new();
        let placeholder = store.insert(
            AssetId::new(".raster/placeholder"),
            Texture::placeholder(gpu, layout),
        );

        Self {
            store,
            project,
            placeholder,
            missing: Vec::new(),
            watcher: Watcher::new(),
        }
    }

    /// Loads a texture, returning the placeholder if it cannot be read.
    ///
    /// Un asset manquant est signale une fois : dans une boucle de jeu, le
    /// signaler a chaque frame noierait la console.
    pub fn load(
        &mut self,
        gpu: &Gpu,
        layout: &wgpu::BindGroupLayout,
        id: &AssetId,
    ) -> Handle<Texture> {
        if let Some(handle) = self.store.handle(id) {
            return handle;
        }

        self.watcher.watch(&self.project, id);

        match self.decode(gpu, layout, id) {
            Ok(texture) => self.store.insert(id.clone(), texture),
            Err(e) => {
                self.report(id, &e);
                // Une entree propre a l'asset, garnie du damier : le handle
                // rendu ici doit rester valide pour que le fichier, une fois
                // apparu, remplace la texture sous les yeux du jeu.
                self.store
                    .insert(id.clone(), Texture::placeholder(gpu, layout))
            }
        }
    }

    /// Reloads the textures whose files changed, returning what was reloaded.
    ///
    /// A appeler a chaque frame : le scrutage s'espace de lui-meme.
    pub fn reload_changed(&mut self, gpu: &Gpu, layout: &wgpu::BindGroupLayout) -> Vec<AssetId> {
        let changed = self.watcher.changed(&self.project);
        self.reload(gpu, layout, &changed)
    }

    /// Reloads the named textures now, whatever the sweep interval says.
    pub fn reload(
        &mut self,
        gpu: &Gpu,
        layout: &wgpu::BindGroupLayout,
        ids: &[AssetId],
    ) -> Vec<AssetId> {
        let mut reloaded = Vec::new();

        for id in ids {
            match self.decode(gpu, layout, id) {
                Ok(texture) => {
                    self.store.insert(id.clone(), texture);
                    // Un asset repare cesse d'etre manquant, et un prochain
                    // echec sera signale a nouveau.
                    self.missing.retain(|m| m != id);
                    reloaded.push(id.clone());
                }
                Err(e) => self.report(id, &e),
            }
        }

        reloaded
    }

    /// The interval between sweeps.
    pub fn set_watch_interval(&mut self, interval: std::time::Duration) {
        self.watcher.set_interval(interval);
    }

    fn decode(
        &self,
        gpu: &Gpu,
        layout: &wgpu::BindGroupLayout,
        id: &AssetId,
    ) -> Result<Texture, TextureError> {
        let bytes = self.project.read(id)?;
        let image = crate::Image::decode(std::io::Cursor::new(bytes))
            .map_err(|source| TextureError::Decode { source })?;
        Ok(Texture::from_image(gpu, layout, &image))
    }

    /// Signale un asset une fois : dans une boucle de jeu, le signaler a chaque
    /// frame noierait la console.
    fn report(&mut self, id: &AssetId, e: &TextureError) {
        if !self.missing.contains(id) {
            eprintln!("{e} — texture de remplacement");
            self.missing.push(id.clone());
        }
    }

    /// Stores a texture built in code rather than read from a file.
    pub fn insert(&mut self, id: AssetId, texture: Texture) -> Handle<Texture> {
        self.store.insert(id, texture)
    }

    #[must_use]
    pub fn get(&self, handle: Handle<Texture>) -> Option<&Texture> {
        self.store.get(handle)
    }

    /// Les textures indexees par `Handle::index`, telles que le batcher les
    /// attend.
    #[must_use]
    pub fn as_slice(&self) -> &[Texture] {
        self.store.as_slice()
    }

    /// The checkerboard shown in place of a missing texture.
    #[must_use]
    pub fn placeholder(&self) -> Handle<Texture> {
        self.placeholder
    }

    #[must_use]
    pub fn project(&self) -> &Project {
        &self.project
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.store.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// The assets that could not be loaded.
    #[must_use]
    pub fn missing(&self) -> &[AssetId] {
        &self.missing
    }
}

/// Why a texture could not be loaded.
#[derive(Debug)]
pub enum TextureError {
    Asset(AssetError),
    Decode { source: crate::ImageError },
}

impl std::fmt::Display for TextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Asset(e) => write!(f, "{e}"),
            Self::Decode { source } => write!(f, "could not decode the image: {source}"),
        }
    }
}

impl std::error::Error for TextureError {}

impl From<AssetError> for TextureError {
    fn from(e: AssetError) -> Self {
        Self::Asset(e)
    }
}
