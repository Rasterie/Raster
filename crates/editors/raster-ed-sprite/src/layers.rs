use raster_ed_visual::Frame;
use raster_math::IVec2;
use raster_render::Colour;

/// How a layer combines with what is under it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Blend {
    /// Le calque recouvre, selon son alpha.
    #[default]
    Normal,
    /// Additionne : ce qui brille sur du sombre.
    Add,
    /// Multiplie : une ombre posee sur ce qui est dessous.
    Multiply,
    /// Ne peint que la ou le dessous est deja opaque.
    Clip,
}

impl Blend {
    pub const ALL: [Self; 4] = [Self::Normal, Self::Add, Self::Multiply, Self::Clip];

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Add => "Addition",
            Self::Multiply => "Multiplication",
            Self::Clip => "Ecretage",
        }
    }
}

/// One layer of a sprite.
#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    pub name: String,
    pub frame: Frame,
    pub blend: Blend,
    /// De 0 a 1.
    pub opacity: f32,
    pub visible: bool,
    /// Un calque verrouille se voit mais ne se modifie pas.
    pub locked: bool,
}

impl Layer {
    #[must_use]
    pub fn new(name: impl Into<String>, width: i32, height: i32) -> Self {
        Self {
            name: name.into(),
            frame: Frame::new(width, height),
            blend: Blend::Normal,
            opacity: 1.0,
            visible: true,
            locked: false,
        }
    }

    /// Whether drawing on this layer is allowed.
    ///
    /// Un calque cache ou verrouille ne se modifie pas : peindre dessus sans
    /// le voir serait pire que de refuser.
    #[must_use]
    pub fn editable(&self) -> bool {
        self.visible && !self.locked
    }
}

/// A sprite: layers, stacked bottom to top.
#[derive(Debug, Clone, PartialEq)]
pub struct Layers {
    layers: Vec<Layer>,
    /// Celui qu'un outil modifie.
    current: usize,
    size: IVec2,
}

impl Layers {
    /// Au-dela, la pile devient impossible a naviguer et le rendu couteux.
    pub const MAX: usize = 32;

    #[must_use]
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            layers: vec![Layer::new("Calque 1", width, height)],
            current: 0,
            size: IVec2::new(width.max(1), height.max(1)),
        }
    }

    #[must_use]
    pub fn size(&self) -> IVec2 {
        self.size
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Une pile de calques n'est jamais vide : il en reste toujours un.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        false
    }

    #[must_use]
    pub fn current(&self) -> &Layer {
        &self.layers[self.current]
    }

    /// The layer a tool draws on, or `None` if it cannot be drawn on.
    pub fn current_mut(&mut self) -> Option<&mut Layer> {
        let layer = &mut self.layers[self.current];
        layer.editable().then_some(layer)
    }

    #[must_use]
    pub fn index(&self) -> usize {
        self.current
    }

    pub fn select(&mut self, index: usize) {
        self.current = index.min(self.layers.len() - 1);
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&Layer> {
        self.layers.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Layer> {
        self.layers.get_mut(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Layer> {
        self.layers.iter()
    }

    /// Adds an empty layer above the current one.
    ///
    /// Renvoie son indice, ou `None` si la pile est pleine.
    pub fn add(&mut self) -> Option<usize> {
        if self.layers.len() >= Self::MAX {
            return None;
        }

        let name = format!("Calque {}", self.layers.len() + 1);
        self.layers
            .insert(self.current + 1, Layer::new(name, self.size.x, self.size.y));
        self.current += 1;
        Some(self.current)
    }

    /// Copies the current layer.
    pub fn duplicate(&mut self) -> Option<usize> {
        if self.layers.len() >= Self::MAX {
            return None;
        }

        let mut copy = self.layers[self.current].clone();
        copy.name = format!("{} (copie)", copy.name);
        self.layers.insert(self.current + 1, copy);
        self.current += 1;
        Some(self.current)
    }

    /// Removes a layer, refusing to leave none.
    pub fn remove(&mut self, index: usize) -> bool {
        if self.layers.len() <= 1 || index >= self.layers.len() {
            return false;
        }

        self.layers.remove(index);
        self.current = self.current.min(self.layers.len() - 1);
        true
    }

    /// Moves a layer up or down the stack.
    ///
    /// L'ordre decide de ce qui recouvre quoi : le deplacer est le geste le
    /// plus courant apres le dessin.
    pub fn move_layer(&mut self, from: usize, to: usize) -> bool {
        if from >= self.layers.len() || to >= self.layers.len() || from == to {
            return false;
        }

        let layer = self.layers.remove(from);
        self.layers.insert(to, layer);

        // La selection suit le calque deplace, pas sa position.
        if self.current == from {
            self.current = to;
        } else if from < self.current && to >= self.current {
            self.current -= 1;
        } else if from > self.current && to <= self.current {
            self.current += 1;
        }

        true
    }

    /// Merges the current layer into the one below.
    ///
    /// Vers le bas : c'est le sens ou l'empilement se lit, et le calque du
    /// dessous garde son nom.
    pub fn merge_down(&mut self) -> bool {
        if self.current == 0 {
            return false;
        }

        let above = self.layers[self.current].clone();
        let below = &mut self.layers[self.current - 1];

        for y in 0..below.frame.height() {
            for x in 0..below.frame.width() {
                let at = IVec2::new(x, y);
                let (Some(under), Some(over)) = (below.frame.get(at), above.frame.get(at)) else {
                    continue;
                };
                below
                    .frame
                    .set(at, blend(under, over, above.blend, above.opacity));
            }
        }

        self.layers.remove(self.current);
        self.current -= 1;
        true
    }

    /// Flattens every visible layer into one frame.
    ///
    /// Ce que l'ecran montre, et ce qu'un export produit.
    #[must_use]
    pub fn flatten(&self) -> Frame {
        let mut out = Frame::new(self.size.x, self.size.y);

        for layer in self.layers.iter().filter(|l| l.visible) {
            for y in 0..self.size.y {
                for x in 0..self.size.x {
                    let at = IVec2::new(x, y);
                    let (Some(under), Some(over)) = (out.get(at), layer.frame.get(at)) else {
                        continue;
                    };
                    out.set(at, blend(under, over, layer.blend, layer.opacity));
                }
            }
        }

        out
    }
}

/// Combines two colours according to a blend mode.
#[must_use]
pub fn blend(under: Colour, over: Colour, mode: Blend, opacity: f32) -> Colour {
    let [ur, ug, ub, ua] = under.to_array();
    let [or_, og, ob, oa] = over.to_array();

    let alpha = (oa * opacity.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    if alpha <= 0.0 {
        return under;
    }

    // L'ecretage ne peint que sur ce qui est deja opaque : c'est ainsi qu'on
    // ombre une forme sans deborder.
    if mode == Blend::Clip && ua <= 0.0 {
        return under;
    }

    let mix = |u: f32, o: f32| match mode {
        Blend::Normal | Blend::Clip => o,
        Blend::Add => (u + o).min(1.0),
        Blend::Multiply => u * o,
    };

    // Composition classique : la couleur du dessous transparait selon l'alpha.
    let out_alpha = alpha + ua * (1.0 - alpha);
    if out_alpha <= 0.0 {
        return Colour::TRANSPARENT;
    }

    let channel = |u: f32, o: f32| (mix(u, o) * alpha + u * ua * (1.0 - alpha)) / out_alpha;

    Colour::rgba(
        channel(ur, or_),
        channel(ug, og),
        channel(ub, ob),
        out_alpha,
    )
}
