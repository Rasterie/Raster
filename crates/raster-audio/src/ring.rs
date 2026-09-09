//! Un tampon circulaire a un producteur et un consommateur.
//!
//! Le jeu ecrit, le thread audio lit, sans verrou : un `Mutex` pris par le
//! callback pendant que le jeu ecrit ferait attendre la carte son, ce qui
//! s'entend comme un craquement.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Une case : `f32` derriere un atomique, car les deux cotes y touchent.
type Sample = std::sync::atomic::AtomicU32;

/// The writing half, held by the game.
pub struct Producer {
    ring: Arc<RingBuffer>,
}

/// The reading half, held by the audio callback.
pub struct Consumer {
    ring: Arc<RingBuffer>,
}

struct RingBuffer {
    samples: Box<[Sample]>,
    write: AtomicUsize,
    read: AtomicUsize,
}

/// Creates a ring holding `capacity` samples, and its two halves.
///
/// # Panics
///
/// Si la capacite est nulle.
#[must_use]
pub fn ring(capacity: usize) -> (Producer, Consumer) {
    assert!(capacity > 0, "a ring buffer cannot be empty");

    let samples = (0..capacity).map(|_| Sample::new(0)).collect();
    let ring = Arc::new(RingBuffer {
        samples,
        write: AtomicUsize::new(0),
        read: AtomicUsize::new(0),
    });

    (Producer { ring: ring.clone() }, Consumer { ring })
}

impl RingBuffer {
    fn capacity(&self) -> usize {
        self.samples.len()
    }

    fn filled(&self) -> usize {
        self.write.load(Ordering::Acquire) - self.read.load(Ordering::Acquire)
    }
}

impl Producer {
    /// Writes what fits, returning how many samples were taken.
    ///
    /// Ce qui deborde est laisse : ecraser ce que le thread audio n'a pas
    /// encore lu produirait un saut dans le son.
    pub fn write(&mut self, samples: &[f32]) -> usize {
        let ring = &*self.ring;
        let capacity = ring.capacity();
        let write = ring.write.load(Ordering::Relaxed);
        let free = capacity - (write - ring.read.load(Ordering::Acquire));
        let n = samples.len().min(free);

        for (i, &sample) in samples[..n].iter().enumerate() {
            ring.samples[(write + i) % capacity].store(sample.to_bits(), Ordering::Relaxed);
        }

        // Release : les echantillons doivent etre visibles avant l'indice qui
        // les annonce, sinon le lecteur lit des cases vides.
        ring.write.store(write + n, Ordering::Release);
        n
    }

    /// How many samples can be written before the ring is full.
    #[must_use]
    pub fn free(&self) -> usize {
        self.ring.capacity() - self.ring.filled()
    }

    #[must_use]
    pub fn filled(&self) -> usize {
        self.ring.filled()
    }

    #[must_use]
    pub fn capacity(&self) -> usize {
        self.ring.capacity()
    }
}

impl Consumer {
    /// Fills `out` with what is available, zeroing the rest.
    ///
    /// Renvoie le nombre d'echantillons reels : le silence qui suit est un
    /// sous-alimentation, que l'appelant peut compter.
    pub fn read(&mut self, out: &mut [f32]) -> usize {
        let ring = &*self.ring;
        let capacity = ring.capacity();
        let read = ring.read.load(Ordering::Relaxed);
        let available = ring.write.load(Ordering::Acquire) - read;
        let n = out.len().min(available);

        for (i, slot) in out[..n].iter_mut().enumerate() {
            *slot = f32::from_bits(ring.samples[(read + i) % capacity].load(Ordering::Relaxed));
        }
        out[n..].fill(0.0);

        ring.read.store(read + n, Ordering::Release);
        n
    }

    #[must_use]
    pub fn filled(&self) -> usize {
        self.ring.filled()
    }

    #[must_use]
    pub fn capacity(&self) -> usize {
        self.ring.capacity()
    }
}
