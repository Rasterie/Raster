# Domain: Audio

Synthesis, cues, mixing, music. Built on **Resonance**, which already exists as
an independent Rust library.

**Crates:** `raster-audio`, `raster-ed-sound`, wrapping `resonance-core`

## What Resonance already provides

Resonance is a working synthesis and sequencing library, not a plan. Its core is
about 2,000 lines with only `cpal` (device output) and `hound` (WAV) as
dependencies.

```
engine/     oscillator, envelope, filter, noise, drum, delay, reverb, stereo
music/      note, sequencer, time
output/     realtime, player, wav
rnc/        a DSL: tokenizer, parser, AST, evaluator
```

Its central abstraction is exactly the right one for an engine to build on:

```rust
pub trait Source {
    fn next_sample(&mut self) -> f32;
}
```

Everything that makes sound implements it, and `Box<dyn Source + Send>` is itself
a `Source`. Composition of sources — the basis of a cue graph — needs no change
to Resonance at all.

The `rnc` DSL matters beyond audio: it is proof that writing a language is within
reach, which is relevant to
[architecture/scripting.md](../architecture/scripting.md).

## The boundary

`resonance-core` stays free of Raster types. It is a library about sound, useful
outside a game engine, and keeping it that way keeps its API honest.

`raster-audio` adds what only a game needs:

| Concern | Where |
| --- | --- |
| Oscillators, envelopes, filters, effects | `resonance-core` |
| Sequencing, notes, timing | `resonance-core` |
| Device output, WAV export | `resonance-core` |
| Cues, buses, mixing | `raster-audio` |
| Spatial attenuation, actor↔sound link | `raster-audio` |
| Asset integration, hot reload | `raster-audio` |

If a change to Raster requires a change to `resonance-core`, that is a signal the
boundary is wrong.

## Cues

A cue is the unit a game plays. Not a file — an assembled behaviour, the direct
analogue of Unreal's Sound Cue.

```rust
ctx.audio.play(asset!("sfx/footstep.cue"));
```

That single call might select one of five samples at random, pitch it by ±5%,
apply an envelope, route it to the SFX bus and attenuate it by distance. All of
that is authored in the cue graph, not written in gameplay code — which is the
entire point. Gameplay says *what happened*; the cue says *what it sounds like*.

A cue graph is built on `raster-graph`, the same node-graph foundation as
material graphs. Node types:

- **Sources** — a sample, a Resonance instrument, noise
- **Selection** — random, sequential, weighted, by parameter
- **Modulation** — pitch, volume, filter, randomised ranges
- **Envelopes and effects** — Resonance's DSP, exposed as nodes
- **Routing** — bus assignment, spatialisation

## Buses and mixing

A small hierarchy — Master, Music, SFX, UI, Ambience — because every game needs
per-category volume, and gameplay should never touch a raw sample.

Buses carry volume, effects, ducking (music dips when dialogue plays) and
snapshots (an underwater mix, a paused mix).

## Spatial audio in 2D

Cheap and effective, and specific to 2D rather than a 3D system reduced:

- **Distance attenuation** with a configurable curve
- **Stereo panning** from horizontal offset relative to the listener
- **Low-pass by distance** — distant sounds lose treble, which does more for
  perceived depth than volume alone
- **Listener** attached to an actor, usually the camera or the player

No HRTF, no reverb zones with geometry. A Terraria-like needs a cave to sound
different from the surface, which is a bus snapshot, not ray-traced acoustics.

## Music

Resonance sequences music rather than only playing files, which allows things a
sample-based engine cannot do:

- **Layered tracks** — instrument layers fading in with intensity
- **Transitions on musical boundaries** — switching at the next bar rather than
  cutting mid-phrase
- **Generated variation** — the same theme, never quite identical

This is optional. Playing an OGG is supported and is what most games will do.
But the capability exists, and for a solo developer who cannot commission a
composer, generated music is a real advantage.

## Editors

**Cue graph** — assemble a cue from nodes, with live preview. The analogue of
Sound Cue.

**Instrument editor** — Resonance's synthesis surface: oscillators, envelopes,
filters, effects. This is where a sound is *created* rather than assembled.

**Sequencer** — tracks, patterns, notes, for music.

All three share the Audio domain's foundations: the transport, the waveform
display, the preview chain. Same reasoning as the Visual domain.

## Threading

Audio runs on its own real-time thread and must never block. No allocation, no
locks, no file I/O on the audio thread — an underrun is an audible click, and
players notice clicks far more than a dropped frame.

The game thread communicates through a lock-free queue: commands in, state out.
Assets are loaded and prepared on the main thread and handed over ready to play.

`cpal` already provides the callback structure Resonance uses. What
`raster-audio` adds is the discipline around it.

## Open questions

- How many voices to support, and the policy when they run out (steal the
  oldest, the quietest, or refuse)
- Whether cue graphs evaluate per-sample or per-block. Per-block is far more
  efficient; per-sample is simpler and matches Resonance's current `next_sample`.
  Probably per-block with Resonance sources adapted.
- Whether `rnc` becomes the authoring format for instruments alongside the visual
  editor, giving both a text and a graph path to the same asset
