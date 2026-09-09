use super::AssetId;
use std::path::{Path, PathBuf};

/// The root a game's assets are named relative to.
///
/// Une scene ne contient jamais de chemin absolu : c'est ce qui rend un projet
/// deplacable d'une machine a l'autre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    root: PathBuf,
}

impl Project {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Looks for a project root at or above `start`, by its marker file.
    ///
    /// Ce qui permet de lancer un jeu depuis n'importe quel sous-dossier.
    #[must_use]
    pub fn discover(start: impl AsRef<Path>) -> Option<Self> {
        let mut current = start.as_ref();
        loop {
            if current.join(Self::MARKER).is_file() {
                return Some(Self::new(current));
            }
            current = current.parent()?;
        }
    }

    /// Le fichier qui marque la racine d'un projet.
    pub const MARKER: &'static str = "raster.toml";

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The path an asset lives at.
    ///
    /// # Errors
    ///
    /// Refuse un identifiant qui sort de la racine : une scene ne doit pas
    /// pouvoir lire un fichier arbitraire de la machine.
    pub fn resolve(&self, id: &AssetId) -> Result<PathBuf, AssetError> {
        if id.is_empty() {
            return Err(AssetError::Empty);
        }
        if id.as_str().starts_with("..") || Path::new(id.as_str()).is_absolute() {
            return Err(AssetError::Escapes(id.clone()));
        }
        Ok(self.root.join(id.as_str()))
    }

    /// The identifier a path inside the project is known by.
    #[must_use]
    pub fn id_of(&self, path: impl AsRef<Path>) -> Option<AssetId> {
        let relative = path.as_ref().strip_prefix(&self.root).ok()?;
        Some(AssetId::new(relative.to_string_lossy()))
    }

    /// Whether the asset exists on disk.
    #[must_use]
    pub fn exists(&self, id: &AssetId) -> bool {
        self.resolve(id).is_ok_and(|path| path.is_file())
    }

    /// Reads an asset's bytes.
    ///
    /// # Errors
    ///
    /// If the identifier is invalid or the file cannot be read.
    pub fn read(&self, id: &AssetId) -> Result<Vec<u8>, AssetError> {
        let path = self.resolve(id)?;
        std::fs::read(&path).map_err(|source| AssetError::Io {
            id: id.clone(),
            source,
        })
    }
}

/// Why an asset could not be resolved or read.
#[derive(Debug)]
pub enum AssetError {
    /// An identifier with no path at all.
    Empty,
    /// Un identifiant qui remonte hors du projet.
    Escapes(AssetId),
    Io {
        id: AssetId,
        source: std::io::Error,
    },
}

impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "the asset path is empty"),
            Self::Escapes(id) => write!(f, "`{id}` points outside the project"),
            Self::Io { id, source } => write!(f, "could not read `{id}`: {source}"),
        }
    }
}

impl std::error::Error for AssetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
