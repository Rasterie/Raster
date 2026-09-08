/// The engine's clock.
///
/// Two deltas rather than one, and the distinction matters: `delta` is scaled
/// by [`Time::scale`], so pausing the game freezes it, while `raw_delta` never
/// is. UI animation runs on `raw_delta` — a pause menu that stops animating
/// when the game pauses looks broken.
#[derive(Debug, Clone, Copy)]
pub struct Time {
    /// Seconds since the last frame, scaled.
    pub delta: f32,
    /// Seconds since the last frame, unscaled. For UI and anything that must
    /// keep moving while the game is paused.
    pub raw_delta: f32,
    /// Seconds since the game started, scaled.
    pub elapsed: f64,
    /// Time multiplier. Zero pauses; slow motion is a game mechanic.
    pub scale: f32,
    /// The fixed step, in seconds.
    pub fixed_delta: f32,
    /// How far through the current fixed step the frame sits, from 0 to 1.
    ///
    /// Rendering interpolates by this so motion stays smooth even though
    /// physics advances in discrete steps.
    pub alpha: f32,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            delta: 0.0,
            raw_delta: 0.0,
            elapsed: 0.0,
            scale: 1.0,
            fixed_delta: Self::DEFAULT_FIXED_DELTA,
            alpha: 0.0,
        }
    }
}

impl Time {
    /// 60 fixed steps a second — the rate most 2D games tune their movement
    /// against.
    pub const DEFAULT_FIXED_DELTA: f32 = 1.0 / 60.0;

    #[must_use]
    pub fn paused(&self) -> bool {
        self.scale == 0.0
    }
}

/// Runs game logic at a fixed rate, whatever the frame rate.
///
/// A platformer stepping physics by the frame delta has a jump height that
/// depends on the machine it runs on. That is discovered far too late, usually
/// by a player on hardware the developer never had, so the engine makes the
/// fixed step the default path rather than an option.
#[derive(Debug, Clone)]
pub struct FrameLoop {
    time: Time,
    /// Time owed to the fixed step but not yet spent.
    accumulator: f32,
    /// The most fixed steps one frame may run.
    max_steps: u32,
}

impl Default for FrameLoop {
    fn default() -> Self {
        Self::new(Time::DEFAULT_FIXED_DELTA)
    }
}

impl FrameLoop {
    /// How many fixed steps a single frame may run before the rest is dropped.
    ///
    /// Without a cap, a frame that took a long time — a breakpoint, a window
    /// drag, a laptop waking up — owes so many steps that running them all
    /// takes even longer, which owes more still. The simulation never catches
    /// up and the game hangs. Better to lose time than to freeze: this is the
    /// "spiral of death", and dropping the excess is the standard answer.
    pub const MAX_STEPS: u32 = 5;

    /// The largest frame delta accepted.
    ///
    /// A frame longer than this almost certainly means the process was
    /// suspended rather than that the game genuinely ran that slowly.
    pub const MAX_FRAME_DELTA: f32 = 0.25;

    #[must_use]
    pub fn new(fixed_delta: f32) -> Self {
        Self {
            time: Time {
                fixed_delta: fixed_delta.max(f32::EPSILON),
                ..Time::default()
            },
            accumulator: 0.0,
            max_steps: Self::MAX_STEPS,
        }
    }

    #[must_use]
    pub fn time(&self) -> &Time {
        &self.time
    }

    /// Sets the time multiplier. Zero pauses.
    pub fn set_scale(&mut self, scale: f32) {
        self.time.scale = scale.max(0.0);
    }

    pub fn set_max_steps(&mut self, steps: u32) {
        self.max_steps = steps.max(1);
    }

    /// Advances by one frame, returning how many fixed steps to run.
    ///
    /// A frame runs its fixed steps first, then its variable update, then
    /// draws — interpolating by [`Time::alpha`].
    pub fn advance(&mut self, raw_delta: f32) -> Steps {
        // Une frame trop longue signale une suspension du processus, pas un
        // ralentissement du jeu : on la traite comme une frame normale plutot
        // que de faire un bond dans la simulation.
        let raw_delta = raw_delta.clamp(0.0, Self::MAX_FRAME_DELTA);

        self.time.raw_delta = raw_delta;
        self.time.delta = raw_delta * self.time.scale;
        self.time.elapsed += f64::from(self.time.delta);

        self.accumulator += self.time.delta;

        /*
          Une tolerance, parce que les f32 s'accumulent mal : trois pas de
          16,667 ms retirés de 50 ms laissent 3,7 ns de moins qu'un pas, et le
          troisieme ne se declencherait jamais. A 20 images/s constantes, la
          simulation perdrait ainsi un pas sur trois et le jeu tournerait au
          ralenti. La tolerance vaut un millieme de pas, soit bien plus que
          l'erreur accumulee et bien moins que ce qui se percoit.
        */
        let seuil = self.time.fixed_delta * (1.0 - 1e-3);

        let mut steps = 0;
        while self.accumulator >= seuil && steps < self.max_steps {
            self.accumulator -= self.time.fixed_delta;
            steps += 1;
        }

        // Le retrait peut passer legerement sous zero a cause de la tolerance.
        self.accumulator = self.accumulator.max(0.0);

        /*
          Le reste depasse encore un pas : la frame a pris trop de temps. On
          abandonne ce qui reste plutot que de le reporter, sinon la dette
          s'accumulerait sans jamais etre rattrapee.
        */
        if self.accumulator >= seuil {
            self.accumulator = 0.0;
        }

        self.time.alpha = self.accumulator / self.time.fixed_delta;

        Steps {
            count: steps,
            fixed_delta: self.time.fixed_delta,
        }
    }
}

/// How many fixed steps this frame owes.
#[derive(Debug, Clone, Copy)]
pub struct Steps {
    count: u32,
    fixed_delta: f32,
}

impl Steps {
    #[must_use]
    pub fn count(self) -> u32 {
        self.count
    }

    #[must_use]
    pub fn fixed_delta(self) -> f32 {
        self.fixed_delta
    }
}

impl Iterator for Steps {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.count == 0 {
            return None;
        }
        self.count -= 1;
        Some(self.fixed_delta)
    }
}
