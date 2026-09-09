use std::collections::HashMap;

/// One image of an animation, with how long it stays on screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    /// Index into the sprite sheet.
    pub index: u32,
    /// Seconds this frame lasts.
    pub duration: f32,
}

impl Frame {
    #[must_use]
    pub fn new(index: u32, duration: f32) -> Self {
        Self {
            index,
            duration: duration.max(f32::EPSILON),
        }
    }
}

/// What happens when an animation reaches its end.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Repeat {
    /// Stops on the last frame.
    Once,
    /// Starts over.
    #[default]
    Loop,
    /// Plays forwards, then backwards, without repeating the end frames.
    PingPong,
}

/// A named sequence of frames.
#[derive(Debug, Clone)]
pub struct Animation {
    pub frames: Vec<Frame>,
    pub repeat: Repeat,
    /// Événements attachés à une image, déclenchés en y entrant.
    ///
    /// Un bruit de pas sur l'image 3, une hitbox sur la 5 : sans cela chaque
    /// jeu recompte le temps de son côté et se désynchronise du dessin.
    events: HashMap<usize, Vec<String>>,
}

impl Animation {
    /// A sequence where every frame lasts the same time.
    ///
    /// # Panics
    ///
    /// If `frames` is empty or `fps` is not positive.
    #[must_use]
    pub fn new(frames: impl IntoIterator<Item = u32>, fps: f32) -> Self {
        assert!(
            fps > 0.0,
            "an animation cannot run at {fps} frames a second"
        );

        let duration = 1.0 / fps;
        let frames: Vec<Frame> = frames
            .into_iter()
            .map(|index| Frame::new(index, duration))
            .collect();

        assert!(!frames.is_empty(), "an animation needs at least one frame");

        Self {
            frames,
            repeat: Repeat::Loop,
            events: HashMap::new(),
        }
    }

    #[must_use]
    pub fn with_repeat(mut self, repeat: Repeat) -> Self {
        self.repeat = repeat;
        self
    }

    /// Attaches an event to a frame, fired when playback enters it.
    #[must_use]
    pub fn with_event(mut self, frame: usize, name: impl Into<String>) -> Self {
        self.events.entry(frame).or_default().push(name.into());
        self
    }

    /// Events attached to a frame.
    #[must_use]
    pub fn events_at(&self, frame: usize) -> &[String] {
        self.events.get(&frame).map_or(&[], Vec::as_slice)
    }

    /// How long one full pass takes.
    #[must_use]
    pub fn duration(&self) -> f32 {
        self.frames.iter().map(|f| f.duration).sum()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}

/// Plays an animation, tracking where it is.
#[derive(Debug, Clone)]
pub struct Animator {
    frame: usize,
    elapsed: f32,
    /// Sens de lecture, pour le ping-pong.
    forwards: bool,
    finished: bool,
    speed: f32,
}

impl Default for Animator {
    fn default() -> Self {
        Self::new()
    }
}

impl Animator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            frame: 0,
            elapsed: 0.0,
            forwards: true,
            finished: false,
            speed: 1.0,
        }
    }

    /// Advances playback, returning the events crossed this step.
    ///
    /// Rend les événements plutôt que de les émettre : le moteur ne sait pas ce
    /// qu'un jeu veut en faire, et un rappel obligerait à le lui passer.
    pub fn advance<'a>(&mut self, animation: &'a Animation, dt: f32) -> Vec<&'a str> {
        let mut fired = Vec::new();
        if animation.is_empty() || self.finished {
            return fired;
        }

        self.elapsed += dt * self.speed;

        // Une boucle : une frame peut en durer moins que le pas de temps.
        while self.elapsed >= animation.frames[self.frame].duration {
            self.elapsed -= animation.frames[self.frame].duration;

            if let Some(next) = self.next_frame(animation) {
                self.frame = next;
                fired.extend(animation.events_at(next).iter().map(String::as_str));
            } else {
                self.finished = true;
                self.elapsed = 0.0;
                break;
            }
        }

        fired
    }

    /// The next frame, or `None` when playback should stop.
    fn next_frame(&mut self, animation: &Animation) -> Option<usize> {
        let last = animation.len() - 1;

        match animation.repeat {
            Repeat::Once if self.frame >= last => None,
            Repeat::Once => Some(self.frame + 1),

            Repeat::Loop => Some((self.frame + 1) % animation.len()),

            Repeat::PingPong if animation.len() == 1 => Some(0),
            Repeat::PingPong => {
                if self.forwards && self.frame >= last {
                    self.forwards = false;
                } else if !self.forwards && self.frame == 0 {
                    self.forwards = true;
                }
                Some(if self.forwards {
                    self.frame + 1
                } else {
                    self.frame - 1
                })
            }
        }
    }

    /// Restarts from the first frame.
    pub fn restart(&mut self) {
        self.frame = 0;
        self.elapsed = 0.0;
        self.forwards = true;
        self.finished = false;
    }

    /// The sprite index to draw.
    #[must_use]
    pub fn index(&self, animation: &Animation) -> u32 {
        animation
            .frames
            .get(self.frame)
            .map_or(0, |frame| frame.index)
    }

    #[must_use]
    pub fn frame(&self) -> usize {
        self.frame
    }

    /// Whether a non-looping animation has reached its end.
    #[must_use]
    pub fn finished(&self) -> bool {
        self.finished
    }

    /// Playback rate; 2.0 plays twice as fast, 0.0 freezes.
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.max(0.0);
    }

    #[must_use]
    pub fn speed(&self) -> f32 {
        self.speed
    }
}
