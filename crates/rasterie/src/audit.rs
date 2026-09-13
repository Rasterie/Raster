//! Les regles du pixel art, appliquees a une image finie.
//!
//! Ce qui distingue un sprite lisible d'une bouillie : des grappes assez
//! grandes pour se voir, aucun pixel isole, et pas de noir pur.

use std::collections::HashMap;

/// How bad a finding is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

/// Something the rules of pixel art object to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub rule: &'static str,
    pub message: String,
    pub severity: Severity,
}

/// En deca, une grappe se lit comme du bruit plutot que comme une forme.
pub const MIN_CLUSTER_SIZE: usize = 10;

/// A grid of colours, as the audit sees it.
///
/// `None` est un pixel vide : il n'appartient a aucune grappe.
pub trait Surface {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    /// La couleur d'un pixel, sous une forme comparable.
    fn colour_at(&self, x: usize, y: usize) -> Option<String>;
}

/// Groups connected pixels of the same colour.
///
/// Connectivite 8 : une diagonale nette est un trait valide en pixel art, pas
/// une suite de pixels isoles.
#[must_use]
pub fn find_clusters(surface: &dyn Surface) -> HashMap<String, Vec<usize>> {
    let (w, h) = (surface.width(), surface.height());
    let mut seen = vec![false; w * h];
    let mut sizes: HashMap<String, Vec<usize>> = HashMap::new();

    for y in 0..h {
        for x in 0..w {
            let index = y * w + x;
            if seen[index] {
                continue;
            }

            let Some(colour) = surface.colour_at(x, y) else {
                seen[index] = true;
                continue;
            };

            let mut size = 0;
            let mut stack = vec![(x as i64, y as i64)];
            seen[index] = true;

            while let Some((cx, cy)) = stack.pop() {
                size += 1;

                for (dx, dy) in [
                    (1, 0),
                    (-1, 0),
                    (0, 1),
                    (0, -1),
                    (1, 1),
                    (-1, -1),
                    (1, -1),
                    (-1, 1),
                ] {
                    let (nx, ny) = (cx + dx, cy + dy);
                    if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
                        continue;
                    }

                    let n_index = ny as usize * w + nx as usize;
                    if seen[n_index] {
                        continue;
                    }
                    if surface.colour_at(nx as usize, ny as usize).as_deref() != Some(&colour) {
                        continue;
                    }

                    seen[n_index] = true;
                    stack.push((nx, ny));
                }
            }

            sizes.entry(colour).or_default().push(size);
        }
    }

    sizes
}

/// Checks an image against the rules of pixel art.
#[must_use]
pub fn audit(surface: &dyn Surface) -> Vec<Finding> {
    let clusters = find_clusters(surface);
    let mut findings = Vec::new();

    let mut orphans = 0;
    let mut small = 0;

    for sizes in clusters.values() {
        for size in sizes {
            if *size == 1 {
                orphans += 1;
            } else if *size < MIN_CLUSTER_SIZE {
                small += 1;
            }
        }
    }

    if orphans > 0 {
        findings.push(Finding {
            rule: "orphan-pixels",
            message: format!(
                "{orphans} pixel{} isole{} detecte{}",
                s(orphans),
                s(orphans),
                s(orphans)
            ),
            severity: Severity::Error,
        });
    }

    if small > 0 {
        findings.push(Finding {
            rule: "cluster-size",
            message: format!(
                "{small} groupe{} sous le seuil de {MIN_CLUSTER_SIZE} pixels",
                s(small)
            ),
            severity: Severity::Warning,
        });
    }

    // Un contour en noir pur ecrase la lumiere : mieux vaut une teinte sombre
    // derivee du corps. Une seule fois : `#000000` n'a que des chiffres, donc
    // aucune autre graphie ne peut a la fois differer et etre reconnue.
    if clusters.keys().any(|c| c.eq_ignore_ascii_case("#000000")) {
        findings.push(Finding {
            rule: "pure-black-outline",
            message: "Contour en noir pur, preferer une teinte sombre derivee du corps".to_owned(),
            severity: Severity::Warning,
        });
    }

    findings
}

/// Le pluriel francais, pour que les messages se lisent.
fn s(n: usize) -> &'static str {
    if n > 1 { "s" } else { "" }
}

/// Whether an image passes without a single error.
#[must_use]
pub fn passes(surface: &dyn Surface) -> bool {
    !audit(surface).iter().any(|f| f.severity == Severity::Error)
}
