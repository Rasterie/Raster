# Scripting

## The position

Gameplay is written in text, in a real programming language. Never in a graph.

Node graphs are used in Raster for **materials** and **sound cues** — dataflow
problems, where a graph genuinely reads better than code. They are not used for
game logic. Unreal's Blueprint spaghetti is the well-known failure mode, and a
solo project has no reason to reproduce it.

## The plan, in order

Raster will eventually support writing gameplay in something friendlier than
Rust. It will not do so early, and this section explains why the order matters
more than the choice.

**Phase 1 — Rust only.** The engine is written and the first game is built in
Rust. No scripting layer exists.

**Phase 2 — an embedded language, to validate the boundary.** Bind an existing
embeddable language (most likely [Rhai](https://rhai.rs), designed for Rust) to
the command surface. The goal is not to ship it as *the* way to write games — it
is to prove the API is genuinely scriptable, and to find everything that is not.

**Phase 3 — a purpose-built language.** Once we know precisely what scripting
needs to express, design a language for it, the way Godot did with GDScript.
This is the intended long-term answer.

**Phase 4 — C#, only if there is demand.** An adoption decision, not a technical
one. See below.

## Why not start with the language

The instinct is to design the scripting language early, since it shapes the API.
That is exactly backwards.

You cannot expose an API that does not exist. Building bindings against a moving
engine means exposing functions that get deleted, and doing the work twice.
Godot's GDScript is good because it was extracted from a working engine, not
designed ahead of one.

What *does* need to happen from the first commit is cheaper and more important:
**design the engine so that scripting is possible later.** Three disciplines,
already recorded in [actors.md](actors.md):

1. **Actors are addressed by `ActorId`.** A script can hold an id. It cannot hold
   a `&mut Player`. If the engine's own API is id-based, scripting adds no new
   constraint.
2. **Every capability goes through `Ctx`.** One explicit, enumerable command
   surface — not free functions, not globals. `Ctx` is the thing a script binds
   to, and knowing that shapes it well from the start.
3. **Fields are reflectable.** A script reads and writes actor properties through
   [reflection](reflection.md) — which the inspector needs anyway.

Hold those three and Phase 2 is an adapter of a few thousand lines. Ignore them
and it is a rewrite of the engine.

## The command surface

`Ctx` is the seam. Everything a script can do, it does through here.

```rust
pub struct Ctx<'w> { /* … */ }

impl<'w> Ctx<'w> {
    // actors
    pub fn spawn<A: Actor>(&mut self, actor: A) -> ActorId;
    pub fn despawn(&mut self, id: ActorId);
    pub fn get(&self, id: ActorId) -> Option<ActorRef<'_>>;
    pub fn send<M: Message>(&mut self, to: ActorId, msg: M);

    // domains, namespaced
    pub fn input(&self) -> &Input;
    pub fn audio(&mut self) -> AudioCtx<'_>;
    pub fn world(&mut self) -> WorldCtx<'_>;
    pub fn time(&self) -> &Time;
}
```

Two properties matter more than the exact shape:

- **It is enumerable.** You can list every command. That list is the scripting
  API, and it can be checked, documented and versioned.
- **It borrows the world once.** Scripts do not hold engine state between calls;
  they receive a context for the duration of a call and give it back.

## On C#

Worth stating plainly, because it comes up first and is usually the wrong first
step.

**What it buys:** a mature language, a large ecosystem, and developers who
already know it. Godot's C# support is a real adoption driver.

**What it costs:** embedding the .NET runtime is a heavy dependency (100+ MB) on
something you do not control; marshalling across the boundary is intricate; a
moving garbage collector next to Rust's ownership model is a genuine source of
subtle bugs; and debugging across the frontier is painful. Godot has spent years
on this and it remains among their most fragile areas.

For a solo project with no users yet, that cost buys nothing. It becomes worth
reconsidering only when there are people asking for it — which is Phase 4, and
which may never arrive.

## On a purpose-built language

This is the intended destination, and it is more plausible here than it would
normally be: Resonance already contains a hand-written language — `rnc`, with a
tokenizer, parser, AST and evaluator in about 900 lines. The skill exists.

A gameplay language is a larger exercise: functions, structs, a type system,
error reporting people can act on, and a bytecode VM rather than a tree-walking
interpreter. Realistically a year of part-time work to reach something pleasant,
plus the surrounding comfort — clear diagnostics, a debugger, editor
autocompletion — without which nobody uses it, including its author.

That is affordable *after* the engine works, and reckless before.

## Hot reload

The reason scripting is wanted at all is iteration speed: changing gameplay
without a Rust recompile.

Worth noting that Rust-side iteration can be improved independently, and should
be, since Phase 1 is Rust-only: fast incremental builds, a dynamically loaded
game library reloaded in place, and preserving world state across a reload using
reflection. That last trick is the same mechanism scripting needs, which is one
more reason reflection comes first.
