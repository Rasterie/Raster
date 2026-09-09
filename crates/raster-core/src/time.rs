/// The engine's clock.
#[derive(Debug, Clone, Copy)]
pub struct Time {
    /// Seconds since the last frame, scaled by [`Time::scale`].
    pub delta: f32,
    /// Seconds since the last frame, unscaled. UI runs on this so menus keep
    /// animating while the game is paused.
    pub raw_delta: f32,
    /// Seconds since the game started, scaled.
    pub elapsed: f64,
    /// Time multiplier. Zero pauses.
    pub scale: f32,
    /// The fixed step, in seconds.
    pub fixed_delta: f32,
    /// How far through the current fixed step this frame sits, 0 to 1.
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
    /// 60 steps a second, the rate most 2D games tune their movement against.
    pub const DEFAULT_FIXED_DELTA: f32 = 1.0 / 60.0;

    #[must_use]
    pub fn paused(&self) -> bool {
        self.scale == 0.0
    }
}

/// Runs game logic at a fixed rate, whatever the frame rate.
///
/// Stepping physics by the frame delta gives a jump height that depends on the
/// machine, which is discovered far too late.
#[derive(Debug, Clone)]
pub struct FrameLoop {
    time: Time,
    /// Time owed to the fixed step but not yet spent.
    accumulator: f32,
    max_steps: u32,
}

impl Default for FrameLoop {
    fn default() -> Self {
        Self::new(Time::DEFAULT_FIXED_DELTA)
    }
}

impl FrameLoop {
    /// Steps one frame may run before the rest is dropped.
    ///
    /// Uncapped, a slow frame owes steps that take longer still to run, and the
    /// simulation never catches up — the spiral of death.
    pub const MAX_STEPS: u32 = 5;

    /// The largest frame delta accepted; beyond it the process was suspended
    /// rather than slow.
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

    /// Advances one frame, returning the fixed steps to run before drawing.
    pub fn advance(&mut self, raw_delta: f32) -> Steps {
        let raw_delta = raw_delta.clamp(0.0, Self::MAX_FRAME_DELTA);

        self.time.raw_delta = raw_delta;
        self.time.delta = raw_delta * self.time.scale;
        self.time.elapsed += f64::from(self.time.delta);
        self.accumulator += self.time.delta;

        // Tolerance : trois pas de 16,667 ms retires de 50 ms laissent, en f32,
        // 3,7 ns de moins qu'un pas, et le troisieme ne partirait jamais.
        let seuil = self.time.fixed_delta * (1.0 - 1e-3);

        let mut steps = 0;
        while self.accumulator >= seuil && steps < self.max_steps {
            self.accumulator -= self.time.fixed_delta;
            steps += 1;
        }
        self.accumulator = self.accumulator.max(0.0);

        // La frame a pris trop de temps : on abandonne le reste plutot que de
        // reporter une dette qui ne serait jamais rattrapee.
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
