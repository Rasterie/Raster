use crate::Vec2;

/// A small, fast, deterministic pseudo-random generator (PCG32).
///
/// Determinism is the point: the same seed always yields the same sequence, on
/// every platform and every run. World generation, procedural sprites and
/// replays all depend on that, and the standard library's generator guarantees
/// none of it.
///
/// Not cryptographically secure, and not intended to be.
#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
    increment: u64,
}

impl Rng {
    /// The multiplier from the reference PCG implementation.
    const MULTIPLIER: u64 = 6_364_136_223_846_793_005;

    /// Two generators with different seeds produce independent sequences.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self::with_stream(seed, 0)
    }

    /// A generator on a distinct stream, so two generators sharing a seed still
    /// produce unrelated sequences — useful when several systems seed from the
    /// same world seed and must not correlate.
    #[must_use]
    pub fn with_stream(seed: u64, stream: u64) -> Self {
        // The increment must be odd for the sequence to reach full period.
        let increment = (stream << 1) | 1;
        let mut rng = Self {
            state: 0,
            increment,
        };
        rng.next_u32();
        rng.state = rng.state.wrapping_add(seed);
        rng.next_u32();
        rng
    }

    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(self.increment);

        // The output permutation: xorshift then a rotation driven by the high
        // bits, which is what gives PCG its statistical quality.
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rotation = (old >> 59) as u32;
        xorshifted.rotate_right(rotation)
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        (u64::from(self.next_u32()) << 32) | u64::from(self.next_u32())
    }

    /// A float in `[0, 1)`.
    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        // 24 bits is exactly f32's mantissa: every value is representable and
        // the distribution stays uniform.
        (self.next_u32() >> 8) as f32 / (1 << 24) as f32
    }

    /// A float in `[min, max)`.
    #[inline]
    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }

    /// An integer in `[min, max)`. Returns `min` when the range is empty.
    #[inline]
    pub fn range_i32(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        let span = (max - min) as u32;
        min + (self.bounded_u32(span) as i32)
    }

    /// An integer in `[0, bound)`, without the modulo bias that `% bound`
    /// would introduce for bounds that do not divide 2^32.
    fn bounded_u32(&mut self, bound: u32) -> u32 {
        if bound == 0 {
            return 0;
        }
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let r = self.next_u32();
            if r >= threshold {
                return r % bound;
            }
        }
    }

    /// `true` with probability `chance`, clamped to `[0, 1]`.
    #[inline]
    pub fn chance(&mut self, chance: f32) -> bool {
        self.next_f32() < chance.clamp(0.0, 1.0)
    }

    #[inline]
    pub fn bool(&mut self) -> bool {
        self.next_u32() & 1 == 1
    }

    /// A point inside the given bounds.
    #[inline]
    pub fn vec2_in(&mut self, min: Vec2, max: Vec2) -> Vec2 {
        Vec2::new(self.range_f32(min.x, max.x), self.range_f32(min.y, max.y))
    }

    /// A uniformly distributed unit vector.
    #[inline]
    pub fn unit_vec2(&mut self) -> Vec2 {
        Vec2::from_angle(self.range_f32(0.0, std::f32::consts::TAU))
    }

    /// A random element, or `None` when the slice is empty.
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            return None;
        }
        items.get(self.bounded_u32(items.len() as u32) as usize)
    }

    /// Shuffles in place, using Fisher-Yates.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.bounded_u32((i + 1) as u32) as usize;
            items.swap(i, j);
        }
    }
}

impl Default for Rng {
    /// A fixed seed, because an unseeded generator that differs between runs
    /// would make a bug impossible to reproduce. Ask for randomness explicitly.
    fn default() -> Self {
        Self::new(0x853c_49e6_748f_ea9b)
    }
}
