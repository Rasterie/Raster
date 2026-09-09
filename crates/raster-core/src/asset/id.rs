use std::fmt;
use std::sync::Arc;

/// A reference to an asset file, relative to the project root.
///
/// Le chemin *est* l'identite : voir `docs/decisions/015-asset-identity.md`.
/// `Arc<str>` parce qu'un identifiant se clone a chaque acteur qui le porte.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AssetId(Arc<str>);

impl AssetId {
    /// Builds an identifier, normalising the path.
    ///
    /// Deux ecritures d'un meme asset doivent donner le meme identifiant, sans
    /// quoi le cache le chargerait deux fois.
    #[must_use]
    pub fn new(path: impl AsRef<str>) -> Self {
        Self(normalise(path.as_ref()).into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The lowercased extension, without the dot.
    #[must_use]
    pub fn extension(&self) -> Option<String> {
        let name = self.0.rsplit('/').next()?;
        let dot = name.rfind('.')?;
        // Un point en tete est un fichier cache, pas une extension.
        if dot == 0 {
            return None;
        }
        Some(name[dot + 1..].to_lowercase())
    }

    /// The file name, extension included.
    #[must_use]
    pub fn file_name(&self) -> &str {
        self.0.rsplit('/').next().unwrap_or(&self.0)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Normalises a path so two spellings of one asset compare equal.
///
/// Sans cela, `./a.png` et `a.png` seraient deux assets, charges deux fois.
fn normalise(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();

    // Un chemin absolu devient `..` : la racine du systeme est hors du projet,
    // et `resolve` refuse ce qui commence par `..`. Sans cela, le `/` de tete
    // disparaitrait et `/etc/passwd` deviendrait un asset du projet.
    if path.starts_with('/') || path.starts_with('\\') || has_drive_prefix(path) {
        parts.push("..");
    }

    for part in path.split(['/', '\\']) {
        match part {
            "" | "." => {}
            // `..` remonte, sauf s'il n'y a plus rien a remonter : le chemin
            // sortirait du projet, on le garde tel quel pour que l'erreur soit
            // visible au chargement plutot que silencieuse ici.
            ".." => {
                if matches!(parts.last(), Some(&last) if last != "..") {
                    parts.pop();
                } else {
                    parts.push("..");
                }
            }
            other => parts.push(other),
        }
    }

    parts.join("/")
}

/// `C:\\...`, qu'un `split` sur les separateurs laisserait passer.
fn has_drive_prefix(path: &str) -> bool {
    let mut chars = path.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic()) && matches!(chars.next(), Some(':'))
}

impl Default for AssetId {
    fn default() -> Self {
        Self::new("")
    }
}

impl fmt::Display for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Affiche le chemin plutot que la structure : `AssetId("a.png")`.
impl fmt::Debug for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AssetId({:?})", &*self.0)
    }
}

impl From<&str> for AssetId {
    fn from(path: &str) -> Self {
        Self::new(path)
    }
}

impl From<String> for AssetId {
    fn from(path: String) -> Self {
        Self::new(path)
    }
}

impl AsRef<str> for AssetId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
