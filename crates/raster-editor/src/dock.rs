use raster_math::Rect;
use raster_ui::layout::Axis;

/// A panel's identity, stable across sessions.
///
/// Une chaine plutot qu'un entier : une disposition enregistree doit rester
/// lisible, et survivre a l'ajout d'un panneau.
pub type PanelId = String;

/// How the shell is divided.
///
/// Un arbre plutot qu'une liste : c'est ce qui permet de fendre un panneau en
/// deux sans toucher a ses voisins.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// Un ou plusieurs panneaux au meme endroit, en onglets.
    Tabs { panels: Vec<PanelId>, active: usize },
    /// Deux enfants separes par une poignee.
    Split {
        axis: Axis,
        /// La part du premier enfant, entre 0 et 1.
        ratio: f32,
        first: Box<Node>,
        second: Box<Node>,
    },
}

impl Node {
    #[must_use]
    pub fn tabs(panels: &[&str]) -> Self {
        Self::Tabs {
            panels: panels.iter().map(|p| (*p).to_owned()).collect(),
            active: 0,
        }
    }

    #[must_use]
    pub fn split(axis: Axis, ratio: f32, first: Node, second: Node) -> Self {
        Self::Split {
            axis,
            ratio: ratio.clamp(0.05, 0.95),
            first: Box::new(first),
            second: Box::new(second),
        }
    }

    /// Every panel this node holds, in order.
    #[must_use]
    pub fn panels(&self) -> Vec<PanelId> {
        match self {
            Self::Tabs { panels, .. } => panels.clone(),
            Self::Split { first, second, .. } => {
                let mut all = first.panels();
                all.extend(second.panels());
                all
            }
        }
    }

    /// Whether a panel lives anywhere in this node.
    #[must_use]
    pub fn contains(&self, panel: &str) -> bool {
        match self {
            Self::Tabs { panels, .. } => panels.iter().any(|p| p == panel),
            Self::Split { first, second, .. } => first.contains(panel) || second.contains(panel),
        }
    }

    /// Removes a panel, collapsing a split that loses its last one.
    ///
    /// Renvoie `None` si le noeud devient vide : c'est au parent de se replier.
    #[must_use]
    pub fn remove(&self, panel: &str) -> Option<Node> {
        match self {
            Self::Tabs { panels, active } => {
                let kept: Vec<PanelId> = panels.iter().filter(|p| *p != panel).cloned().collect();
                if kept.is_empty() {
                    return None;
                }
                Some(Self::Tabs {
                    active: (*active).min(kept.len() - 1),
                    panels: kept,
                })
            }
            Self::Split {
                axis,
                ratio,
                first,
                second,
            } => match (first.remove(panel), second.remove(panel)) {
                (Some(a), Some(b)) => Some(Self::split(*axis, *ratio, a, b)),
                // Un enfant vide : le survivant remonte a la place du parent.
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
        }
    }
}

/// Where each panel sits on screen.
#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    pub panel: PanelId,
    pub area: Rect,
    /// Les autres panneaux du meme groupe d'onglets.
    pub siblings: Vec<PanelId>,
    /// La barre d'onglets, vide quand le panneau est seul.
    pub tab_bar: Option<Rect>,
}

/// The panel arrangement.
#[derive(Debug, Clone, PartialEq)]
pub struct Dock {
    pub root: Node,
    /// La hauteur d'une barre d'onglets.
    pub tab_height: f32,
    /// L'epaisseur d'une poignee de separation.
    pub handle: f32,
}

impl Dock {
    #[must_use]
    pub fn new(root: Node) -> Self {
        Self {
            root,
            tab_height: 18.0,
            handle: 4.0,
        }
    }

    /// Computes where every visible panel goes.
    ///
    /// Seul l'onglet actif de chaque groupe est place : les autres ne sont pas
    /// dessines, donc n'ont pas de rectangle.
    #[must_use]
    pub fn layout(&self, area: Rect) -> Vec<Placement> {
        let mut out = Vec::new();
        self.place(&self.root, area, &mut out);
        out
    }

    fn place(&self, node: &Node, area: Rect, out: &mut Vec<Placement>) {
        match node {
            Node::Tabs { panels, active } => {
                let Some(panel) = panels.get(*active).or_else(|| panels.first()) else {
                    return;
                };

                // Une barre d'onglets n'apparait qu'a plusieurs.
                let (tab_bar, body) = if panels.len() > 1 {
                    let bar = Rect::new(
                        area.position.x,
                        area.position.y,
                        area.size.x,
                        self.tab_height,
                    );
                    let body = Rect::new(
                        area.position.x,
                        area.position.y + self.tab_height,
                        area.size.x,
                        (area.size.y - self.tab_height).max(0.0),
                    );
                    (Some(bar), body)
                } else {
                    (None, area)
                };

                out.push(Placement {
                    panel: panel.clone(),
                    area: body,
                    siblings: panels.clone(),
                    tab_bar,
                });
            }
            Node::Split {
                axis,
                ratio,
                first,
                second,
            } => {
                let (a, b) = self.divide(area, *axis, *ratio);
                self.place(first, a, out);
                self.place(second, b, out);
            }
        }
    }

    /// Splits an area, leaving room for the handle between the two halves.
    #[must_use]
    pub fn divide(&self, area: Rect, axis: Axis, ratio: f32) -> (Rect, Rect) {
        let ratio = ratio.clamp(0.0, 1.0);

        match axis {
            Axis::Horizontal => {
                let usable = (area.size.x - self.handle).max(0.0);
                let left = usable * ratio;
                (
                    Rect::new(area.position.x, area.position.y, left, area.size.y),
                    Rect::new(
                        area.position.x + left + self.handle,
                        area.position.y,
                        usable - left,
                        area.size.y,
                    ),
                )
            }
            Axis::Vertical => {
                let usable = (area.size.y - self.handle).max(0.0);
                let top = usable * ratio;
                (
                    Rect::new(area.position.x, area.position.y, area.size.x, top),
                    Rect::new(
                        area.position.x,
                        area.position.y + top + self.handle,
                        area.size.x,
                        usable - top,
                    ),
                )
            }
        }
    }

    /// The handle between two children of a split.
    #[must_use]
    pub fn handle_area(&self, area: Rect, axis: Axis, ratio: f32) -> Rect {
        let (first, _) = self.divide(area, axis, ratio);
        match axis {
            Axis::Horizontal => Rect::new(
                area.position.x + first.size.x,
                area.position.y,
                self.handle,
                area.size.y,
            ),
            Axis::Vertical => Rect::new(
                area.position.x,
                area.position.y + first.size.y,
                area.size.x,
                self.handle,
            ),
        }
    }

    /// Removes a panel from the arrangement.
    pub fn close(&mut self, panel: &str) -> bool {
        match self.root.remove(panel) {
            Some(root) => {
                let changed = root != self.root;
                self.root = root;
                changed
            }
            // Fermer le dernier panneau laisserait un ecran vide : refuse.
            None => false,
        }
    }

    #[must_use]
    pub fn panels(&self) -> Vec<PanelId> {
        self.root.panels()
    }
}
