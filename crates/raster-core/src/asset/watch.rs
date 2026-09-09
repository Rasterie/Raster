use super::{AssetId, Project};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};

/// Watches loaded assets for changes on disk.
///
/// Par scrutage plutot que par evenements du systeme : voir
/// `docs/decisions/016-hot-reload-polling.md`.
pub struct Watcher {
    stamps: HashMap<AssetId, Option<Stamp>>,
    interval: Duration,
    last: Option<Instant>,
}

impl Watcher {
    /// Quatre balayages par seconde : imperceptible a la main, negligeable au
    /// profilage.
    pub const DEFAULT_INTERVAL: Duration = Duration::from_millis(250);

    #[must_use]
    pub fn new() -> Self {
        Self::with_interval(Self::DEFAULT_INTERVAL)
    }

    #[must_use]
    pub fn with_interval(interval: Duration) -> Self {
        Self {
            stamps: HashMap::new(),
            interval,
            last: None,
        }
    }

    /// Starts watching an asset, recording the state it was loaded in.
    ///
    /// Un fichier absent est surveille aussi : il apparaitra comme un
    /// changement des qu'il existera.
    pub fn watch(&mut self, project: &Project, id: &AssetId) {
        let stamp = modified(project, id);
        self.stamps.insert(id.clone(), stamp);
    }

    /// Changes the sweep interval, keeping what is already watched.
    pub fn set_interval(&mut self, interval: Duration) {
        self.interval = interval;
    }

    #[must_use]
    pub fn interval(&self) -> Duration {
        self.interval
    }

    pub fn forget(&mut self, id: &AssetId) {
        self.stamps.remove(id);
    }

    #[must_use]
    pub fn is_watching(&self, id: &AssetId) -> bool {
        self.stamps.contains_key(id)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.stamps.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stamps.is_empty()
    }

    /// The assets that changed since the last sweep, if one is due.
    ///
    /// Renvoie une liste vide tant que l'intervalle n'est pas ecoule : appeler
    /// a chaque frame est prevu.
    pub fn changed(&mut self, project: &Project) -> Vec<AssetId> {
        let now = Instant::now();
        if let Some(last) = self.last
            && now.duration_since(last) < self.interval
        {
            return Vec::new();
        }
        self.last = Some(now);
        self.sweep(project)
    }

    /// Sweeps now, whatever the interval says.
    pub fn sweep(&mut self, project: &Project) -> Vec<AssetId> {
        let mut changed = Vec::new();

        for (id, stamp) in &mut self.stamps {
            let current = modified(project, id);
            if current != *stamp {
                *stamp = current;
                changed.push(id.clone());
            }
        }

        // Trie : deux balayages d'un meme changement doivent donner le meme
        // ordre, sans quoi un test dependrait du hachage.
        changed.sort();
        changed
    }
}

/// Ce qui distingue deux versions d'un fichier.
///
/// La taille accompagne la date car sa granularite varie selon le systeme de
/// fichiers : une seconde sur certains, ou deux enregistrements rapproches
/// porteraient la meme date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Stamp {
    modified: SystemTime,
    size: u64,
}

/// L'etat du fichier, `None` s'il est illisible ou absent.
fn modified(project: &Project, id: &AssetId) -> Option<Stamp> {
    let path = project.resolve(id).ok()?;
    let meta = std::fs::metadata(path).ok()?;
    Some(Stamp {
        modified: meta.modified().ok()?,
        size: meta.len(),
    })
}

impl Default for Watcher {
    fn default() -> Self {
        Self::new()
    }
}
