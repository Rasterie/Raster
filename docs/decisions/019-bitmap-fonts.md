# 019 — Bitmap fonts: one built in, others loaded

**Status:** Accepted
**Date:** 2026-09-09

## Context

`docs/friction.md` entry 2 records that the MVP could not write "Press Enter" on
its own title screen. Text is the first thing M4 owes, and every widget after it
depends on the answer.

`docs/domains/visual.md` already settles the *kind* of font: bitmap first,
because a pixel art engine needs pixel fonts more than it needs hinted vector
outlines. What is open is where the first one comes from.

## Options

**A. Embed a licensed bitmap font.** A `.png` atlas plus metrics, from one of
the free pixel fonts. Immediately good-looking, and adds a licence to track, an
asset to ship, and a file format to define before anything can draw a letter.

**B. Generate a small font in code.** A 5×7 glyph per character, described as
bitmask rows. No dependency, no licence, no asset — and it works before the
asset pipeline is involved at all.

## Decision

**Both, in order: B now, A as a format later.**

The engine ships one built-in 5×7 font, defined in code, always available. It
covers ASCII 32–126, which is what a HUD, a menu and a debug overlay need.

A loadable font — an atlas plus per-glyph metrics — comes after, as an asset
type. That is when kerning, variable width and non-ASCII belong.

The reason for the order is that a font in code has no failure mode. It cannot
be missing, cannot fail to decode, and cannot be forgotten in a release archive.
Every widget built on top of it therefore works in a test, in a headless build,
and in an editor that has not loaded a project yet. A loaded font is strictly
more capable and strictly more fragile; the built-in one is the floor that never
falls through.

## Consequences

- `raster-ui` can draw text from its first commit, without an asset pipeline.
- The built-in font is deliberately plain. It is a floor, not a house style.
- Glyphs are 5×7 in a 6×8 cell — one column and one row of spacing baked in, so
  text lays out by multiplying, with no per-glyph metrics to consult.
- A `Font` trait, or an enum with a built-in and a loaded variant, keeps the
  call sites identical when loaded fonts arrive.

## Revisiting

If the built-in font turns out to be what everyone ships with — which would mean
the loaded-font path is not pulling its weight — that is worth knowing, and is an
argument for investing in the built-in one rather than replacing it.
