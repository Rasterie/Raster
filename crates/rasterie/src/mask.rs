/// A grid of on/off cells: the shape before it has a colour.
///
/// Ce que la grammaire produit et que le rendu consomme : separer les deux
/// permet de tester une forme sans parler de couleur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mask {
    width: usize,
    height: usize,
    cells: Vec<bool>,
}

impl Mask {
    /// # Panics
    ///
    /// Si une dimension est nulle : un masque vide masquerait une erreur de
    /// calcul plus haut.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width > 0 && height > 0, "a mask cannot be {width}x{height}");

        Self {
            width,
            height,
            cells: vec![false; width * height],
        }
    }

    #[must_use]
    pub fn width(&self) -> usize {
        self.width
    }

    #[must_use]
    pub fn height(&self) -> usize {
        self.height
    }

    /// Whether a cell is set. Hors cadre est toujours vide, ce qui evite un
    /// test de bornes a chaque voisin.
    #[must_use]
    pub fn is_inside(&self, x: i64, y: i64) -> bool {
        if x < 0 || y < 0 || x >= self.width as i64 || y >= self.height as i64 {
            return false;
        }
        self.cells[y as usize * self.width + x as usize]
    }

    /// Sets a cell, ignoring anything outside.
    pub fn put(&mut self, x: i64, y: i64, on: bool) {
        if x < 0 || y < 0 || x >= self.width as i64 || y >= self.height as i64 {
            return;
        }
        self.cells[y as usize * self.width + x as usize] = on;
    }

    #[must_use]
    pub fn cells(&self) -> &[bool] {
        &self.cells
    }

    /// How many cells are set.
    #[must_use]
    pub fn count(&self) -> usize {
        self.cells.iter().filter(|c| **c).count()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// The mask as text, for reading a shape in a test.
    #[must_use]
    pub fn to_text(&self) -> String {
        (0..self.height)
            .map(|y| {
                (0..self.width)
                    .map(|x| {
                        if self.cells[y * self.width + x] {
                            '#'
                        } else {
                            '.'
                        }
                    })
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
