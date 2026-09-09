use raster_render::Colour;

/// The visual identity, in one place.
///
/// Un jeu change ces valeurs et toute son interface suit : c'est ce qui evite
/// que chaque widget porte ses propres couleurs en dur.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    pub background: Colour,
    pub surface: Colour,
    /// Le fond d'un element survole.
    pub hovered: Colour,
    /// Celui d'un element enfonce.
    pub pressed: Colour,
    pub border: Colour,
    /// La bordure d'un element qui a le focus clavier.
    pub focus: Colour,
    pub text: Colour,
    /// Un texte secondaire, moins appuye.
    pub muted: Colour,
    /// Le texte d'un element desactive.
    pub disabled: Colour,
    pub accent: Colour,

    /// L'espace entre deux elements.
    pub gap: f32,
    /// La marge interieure d'un element.
    pub padding: f32,
    /// La hauteur d'un bouton ou d'un champ.
    pub row_height: f32,
    /// L'echelle du texte.
    pub text_scale: f32,
}

impl Theme {
    /// Le theme sombre par defaut : du pixel art sur fond fonce.
    #[must_use]
    pub const fn dark() -> Self {
        Self {
            background: Colour::rgb(0.055, 0.05, 0.085),
            surface: Colour::rgb(0.13, 0.12, 0.18),
            hovered: Colour::rgb(0.20, 0.19, 0.27),
            pressed: Colour::rgb(0.10, 0.09, 0.14),
            border: Colour::rgb(0.30, 0.29, 0.38),
            focus: Colour::rgb(0.55, 0.72, 1.0),
            text: Colour::rgb(0.92, 0.92, 0.96),
            muted: Colour::rgb(0.62, 0.63, 0.72),
            disabled: Colour::rgb(0.38, 0.38, 0.45),
            accent: Colour::rgb(0.93, 0.78, 0.38),

            gap: 4.0,
            padding: 4.0,
            row_height: 16.0,
            text_scale: 1.0,
        }
    }

    /// Un theme clair, pour un editeur ou un jeu qui n'est pas sombre.
    #[must_use]
    pub const fn light() -> Self {
        Self {
            background: Colour::rgb(0.90, 0.90, 0.92),
            surface: Colour::rgb(0.97, 0.97, 0.98),
            hovered: Colour::rgb(0.87, 0.88, 0.92),
            pressed: Colour::rgb(0.80, 0.81, 0.86),
            border: Colour::rgb(0.68, 0.69, 0.74),
            focus: Colour::rgb(0.20, 0.42, 0.85),
            text: Colour::rgb(0.10, 0.10, 0.14),
            muted: Colour::rgb(0.40, 0.41, 0.48),
            disabled: Colour::rgb(0.65, 0.65, 0.70),
            accent: Colour::rgb(0.78, 0.52, 0.10),
            ..Self::dark()
        }
    }

    /// Le fond d'un element selon ce qui lui arrive.
    #[must_use]
    pub fn surface_for(&self, state: State) -> Colour {
        match state {
            State::Pressed => self.pressed,
            State::Hovered => self.hovered,
            _ => self.surface,
        }
    }

    /// La couleur du texte selon l'etat.
    #[must_use]
    pub fn text_for(&self, state: State) -> Colour {
        if state == State::Disabled {
            self.disabled
        } else {
            self.text
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

/// What is happening to a widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum State {
    #[default]
    Idle,
    Hovered,
    Pressed,
    Disabled,
}

impl State {
    #[must_use]
    pub fn interactive(self) -> bool {
        self != Self::Disabled
    }
}
