use raster_core::asset::{AssetId, Project};
use std::path::Path;

/// One entry in the asset tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub id: AssetId,
    /// Le nom affiche, sans le chemin.
    pub name: String,
    pub kind: Kind,
    /// La profondeur dans l'arbre, pour l'indentation.
    pub depth: usize,
}

/// What an asset is, from its extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Folder,
    Sprite,
    Scene,
    Sound,
    /// Un fichier que l'editeur ne sait pas ouvrir : montre, jamais cache.
    Other,
}

impl Kind {
    /// The kind an extension implies.
    #[must_use]
    pub fn of(id: &AssetId) -> Self {
        match id.extension().as_deref() {
            Some("png" | "bmp" | "jpg" | "jpeg") => Self::Sprite,
            Some("toml") if id.as_str().ends_with(".scene.toml") => Self::Scene,
            Some("wav" | "ogg") => Self::Sound,
            _ => Self::Other,
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Folder => "dossier",
            Self::Sprite => "sprite",
            Self::Scene => "scene",
            Self::Sound => "son",
            Self::Other => "fichier",
        }
    }
}

/// The project's assets, as a flat list with depth.
///
/// Plate plutot qu'imbriquee : c'est ce qu'une liste defilante consomme, et
/// l'indentation suffit a montrer la hierarchie.
#[derive(Debug, Clone, Default)]
pub struct Browser {
    entries: Vec<Entry>,
    /// Ce que la recherche laisse passer, `None` quand elle est vide.
    filter: Option<String>,
    /// Les dossiers replies : leur contenu n'est pas liste.
    collapsed: Vec<AssetId>,
}

impl Browser {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Walks a project and records every asset.
    ///
    /// # Errors
    ///
    /// If the project root cannot be read.
    pub fn scan(&mut self, project: &Project) -> std::io::Result<()> {
        self.entries.clear();
        walk(project, project.root(), 0, &mut self.entries)?;

        // Dossiers d'abord, puis par nom : un arbre trie au hasard est
        // impraticable.
        self.entries.sort_by(|a, b| {
            a.id.as_str()
                .rsplit_once('/')
                .map(|(dir, _)| dir.to_owned())
                .unwrap_or_default()
                .cmp(
                    &b.id
                        .as_str()
                        .rsplit_once('/')
                        .map(|(dir, _)| dir.to_owned())
                        .unwrap_or_default(),
                )
                .then(b.kind.cmp(&a.kind))
                .then(a.id.as_str().cmp(b.id.as_str()))
        });
        Ok(())
    }

    /// Every entry, filtered and with collapsed folders' contents hidden.
    #[must_use]
    pub fn visible(&self) -> Vec<&Entry> {
        self.entries
            .iter()
            .filter(|e| self.matches(e) && !self.is_hidden(e))
            .collect()
    }

    fn matches(&self, entry: &Entry) -> bool {
        match &self.filter {
            // Insensible a la casse : personne ne tape les majuscules.
            Some(needle) => entry.id.as_str().to_lowercase().contains(needle),
            None => true,
        }
    }

    /// Whether a collapsed folder hides this entry.
    fn is_hidden(&self, entry: &Entry) -> bool {
        // Une recherche montre tout ce qui correspond, repli ou non : sinon on
        // chercherait sans trouver.
        if self.filter.is_some() {
            return false;
        }

        self.collapsed.iter().any(|folder| {
            let prefix = format!("{}/", folder.as_str());
            entry.id.as_str().starts_with(&prefix)
        })
    }

    /// Sets the search text; empty clears it.
    pub fn search(&mut self, needle: &str) {
        let trimmed = needle.trim().to_lowercase();
        self.filter = (!trimmed.is_empty()).then_some(trimmed);
    }

    #[must_use]
    pub fn filter(&self) -> Option<&str> {
        self.filter.as_deref()
    }

    /// Folds a folder, or unfolds it.
    pub fn toggle(&mut self, folder: &AssetId) {
        if let Some(i) = self.collapsed.iter().position(|f| f == folder) {
            self.collapsed.remove(i);
        } else {
            self.collapsed.push(folder.clone());
        }
    }

    #[must_use]
    pub fn is_collapsed(&self, folder: &AssetId) -> bool {
        self.collapsed.contains(folder)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Every asset of one kind.
    #[must_use]
    pub fn of_kind(&self, kind: Kind) -> Vec<&Entry> {
        self.entries.iter().filter(|e| e.kind == kind).collect()
    }
}

/// Le fichier marqueur et les dossiers de service n'ont rien a faire dans un
/// navigateur d'assets.
fn skip(name: &str) -> bool {
    name.starts_with('.') || name == "target" || name == Project::MARKER
}

fn walk(project: &Project, dir: &Path, depth: usize, out: &mut Vec<Entry>) -> std::io::Result<()> {
    let mut children: Vec<_> = std::fs::read_dir(dir)?.filter_map(Result::ok).collect();
    children.sort_by_key(std::fs::DirEntry::file_name);

    for child in children {
        let name = child.file_name().to_string_lossy().into_owned();
        if skip(&name) {
            continue;
        }

        let path = child.path();
        let Some(id) = project.id_of(&path) else {
            continue;
        };

        let is_dir = path.is_dir();
        out.push(Entry {
            name,
            kind: if is_dir { Kind::Folder } else { Kind::of(&id) },
            id,
            depth,
        });

        if is_dir {
            walk(project, &path, depth + 1, out)?;
        }
    }

    Ok(())
}
