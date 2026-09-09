use crate::{Gpu, Texture};
use raster_core::asset::{AssetError, AssetId, AssetStore, Handle, Project};

/// Textures loaded from a project, each one uploaded once.
///
/// Le damier remplace toute texture manquante : un asset absent doit se voir a
/// l'ecran, pas empecher le jeu de demarrer.
pub struct Textures {
    store: AssetStore<Texture>,
    project: Project,
    placeholder: Handle<Texture>,
    missing: Vec<AssetId>,
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
        let project = &self.project;
        let result = self.store.load_with(id, |id| {
            let bytes = project.read(id)?;
            let image = crate::Image::decode(std::io::Cursor::new(bytes))
                .map_err(|source| TextureError::Decode { source })?;
            Ok::<_, TextureError>(Texture::from_image(gpu, layout, &image))
        });

        match result {
            Ok(handle) => handle,
            Err(e) => {
                if !self.missing.contains(id) {
                    eprintln!("{e} — texture de remplacement");
                    self.missing.push(id.clone());
                }
                self.placeholder
            }
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
