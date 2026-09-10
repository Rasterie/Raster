use crate::dock::{Dock, Node};
use raster_ui::layout::Axis;
use std::path::{Path, PathBuf};

/// What the editor remembers between sessions.
///
/// Par projet plutot que global : deux projets n'ont pas la meme disposition,
/// et la partager forcerait a la refaire a chaque changement.
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub dock: Dock,
    /// La derniere scene ouverte, rouverte au demarrage.
    pub scene: Option<String>,
    pub snap: bool,
    pub show_grid: bool,
    pub zoom: u32,
}

impl Session {
    /// Le fichier ou la session vit, dans le projet.
    pub const FILE: &'static str = ".raster-session.toml";

    #[must_use]
    pub fn new(dock: Dock) -> Self {
        Self {
            dock,
            scene: None,
            snap: true,
            show_grid: true,
            zoom: 2,
        }
    }

    /// Writes the session as TOML.
    ///
    /// Ecrit a la main plutot que par serde : la disposition est un arbre, et
    /// une seule crate de plus ne se justifie pas pour six champs.
    #[must_use]
    pub fn to_toml(&self) -> String {
        let mut out = String::from("[session]\n");
        out.push_str(&format!("snap = {}\n", self.snap));
        out.push_str(&format!("grid = {}\n", self.show_grid));
        out.push_str(&format!("zoom = {}\n", self.zoom));

        if let Some(scene) = &self.scene {
            out.push_str(&format!("scene = {scene:?}\n"));
        }

        out.push_str("\n[layout]\n");
        out.push_str(&format!("tree = {:?}\n", encode(&self.dock.root)));
        out
    }

    /// Reads a session back, falling back to `default` for anything missing.
    ///
    /// Une session illisible ne doit pas empecher d'ouvrir un projet : on
    /// repart de la disposition par defaut.
    #[must_use]
    pub fn from_toml(text: &str, default: Dock) -> Self {
        let mut session = Self::new(default);

        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let (key, value) = (key.trim(), value.trim());

            match key {
                "snap" => session.snap = value == "true",
                "grid" => session.show_grid = value == "true",
                "zoom" => {
                    if let Ok(z) = value.parse::<u32>() {
                        session.zoom = z.clamp(1, 16);
                    }
                }
                "scene" => session.scene = Some(unquote(value)),
                "tree" => {
                    if let Some(root) = decode(&unquote(value)) {
                        session.dock = Dock::new(root);
                    }
                }
                _ => {}
            }
        }

        session
    }

    /// # Errors
    ///
    /// If the file cannot be written.
    pub fn save(&self, root: &Path) -> std::io::Result<PathBuf> {
        let path = root.join(Self::FILE);
        std::fs::write(&path, self.to_toml())?;
        Ok(path)
    }

    /// Loads a session, or the default one if there is none.
    #[must_use]
    pub fn load(root: &Path, default: Dock) -> Self {
        match std::fs::read_to_string(root.join(Self::FILE)) {
            Ok(text) => Self::from_toml(&text, default),
            Err(_) => Self::new(default),
        }
    }
}

fn unquote(value: &str) -> String {
    value.trim_matches('"').to_owned()
}

/// La disposition en une ligne : `H:0.2(assets|V:0.8(scene|console))`.
///
/// Un format compact plutot qu'imbrique en TOML : un arbre s'y ecrit mal, et
/// celui-ci reste lisible dans un fichier ouvert a la main.
fn encode(node: &Node) -> String {
    match node {
        Node::Tabs { panels, active } => {
            format!("[{}@{active}]", panels.join(","))
        }
        Node::Split {
            axis,
            ratio,
            first,
            second,
        } => {
            let a = if *axis == Axis::Horizontal { 'H' } else { 'V' };
            format!("{a}:{ratio:.3}({}|{})", encode(first), encode(second))
        }
    }
}

/// Reads back what `encode` wrote.
///
/// `None` sur une entree malformee : mieux vaut la disposition par defaut
/// qu'un arbre a moitie construit.
fn decode(text: &str) -> Option<Node> {
    let text = text.trim();

    if let Some(inner) = text.strip_prefix('[').and_then(|t| t.strip_suffix(']')) {
        let (names, active) = inner.rsplit_once('@')?;
        let panels: Vec<String> = names
            .split(',')
            .filter(|n| !n.is_empty())
            .map(str::to_owned)
            .collect();

        if panels.is_empty() {
            return None;
        }
        let active = active.parse::<usize>().ok()?.min(panels.len() - 1);
        return Some(Node::Tabs { panels, active });
    }

    let (head, rest) = text.split_once(':')?;
    let axis = match head {
        "H" => Axis::Horizontal,
        "V" => Axis::Vertical,
        _ => return None,
    };

    let (ratio, inner) = rest.split_once('(')?;
    let inner = inner.strip_suffix(')')?;
    let ratio = ratio.parse::<f32>().ok()?;

    // La barre qui separe les deux enfants est celle du niveau courant, pas
    // celle d'un enfant : on compte les parentheses pour la trouver.
    let mut depth = 0;
    let split = inner.char_indices().find(|(_, c)| {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            '|' if depth == 0 => return true,
            _ => {}
        }
        false
    })?;

    let first = decode(&inner[..split.0])?;
    let second = decode(&inner[split.0 + 1..])?;
    Some(Node::split(axis, ratio, first, second))
}
