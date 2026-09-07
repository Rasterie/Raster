# Reflection

Reflection is the ability to inspect and manipulate types at runtime without
knowing them at compile time: to list an actor's fields, read one by name, write
one from a string, and enumerate what types exist.

It is unglamorous, and it is the single most load-bearing mechanism in Raster.

## Why it comes first

Four features that look unrelated all reduce to the same requirement:

| Feature | What it needs |
| --- | --- |
| The inspector | List a selected actor's fields, their types, and edit them |
| Scene serialisation | Write every field to text and read it back |
| Scripting | Read and write actor properties from another language |
| Hot reload | Preserve state across a reload by name |

Build reflection once and all four become tractable. Skip it and each is
implemented ad hoc, three times, incompatibly. This is the classic engine
mistake, and it is why reflection is Wave 1 work rather than "later, when we
need it".

## The shape

A derive macro generates a static description of each actor type.

```rust
#[derive(Actor)]
pub struct Player {
    sprite: Sprite,
    body: Body,

    #[property(min = 0.0, max = 500.0)]
    speed: f32,

    #[property(skip)]
    internal_cache: Vec<u32>,
}
```

From which:

```rust
pub struct TypeInfo {
    pub name: &'static str,
    pub fields: &'static [FieldInfo],
}

pub struct FieldInfo {
    pub name: &'static str,
    pub kind: ValueKind,
    pub offset: usize,
    pub attrs: PropertyAttrs,
}
```

`ValueKind` covers the primitives, the engine's own types (`Vec2`, `Color`,
`Rect`, `AssetId`, `ActorId`), components, enums with named variants, and
sequences of those. Deliberately not: arbitrary user types with no registration,
closures, trait objects.

## The value model

Dynamic access needs a common currency.

```rust
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Vec2(Vec2),
    Color(Color),
    Rect(Rect),
    Asset(AssetIdRaw),
    Actor(ActorId),
    List(Vec<Value>),
    Struct(HashMap<String, Value>),
    Enum { variant: String, payload: Box<Value> },
}
```

Every reflected read yields a `Value`; every reflected write takes one. The
inspector edits `Value`s. The serialiser writes `Value`s. A script exchanges
`Value`s. One conversion layer, not four.

## Attributes

Attributes on a field control how it is treated across all four consumers:

| Attribute | Effect |
| --- | --- |
| `#[property]` | Explicitly exposed (the default for public-shaped fields) |
| `#[property(skip)]` | Hidden from inspector, serialisation and scripting |
| `#[property(min, max)]` | Inspector renders a slider; values are clamped |
| `#[property(readonly)]` | Displayed but not editable |
| `#[property(rename = "…")]` | Serialised name differs from the field name |
| `#[property(tooltip = "…")]` | Inspector help text |

`rename` matters more than it looks: it is what lets a field be renamed in code
without invalidating every scene file that references it.

## Registration

Types must be discoverable by name at runtime — to instantiate a `Player` from a
scene file, the engine needs a name-to-constructor map.

The derive macro emits a registration entry collected at startup. Whether this
uses a linker-section crate like `inventory` or an explicit registration call in
the game's entry point is undecided; the explicit version is uglier and far more
predictable, and predictability probably wins.

## What reflection does not do

- **It does not make the engine dynamic.** Rust code still accesses fields
  directly and statically. Reflection is a parallel path used by tools, not the
  normal way to write gameplay.
- **It does not cost at runtime.** `TypeInfo` is static data. Reflected access
  is only paid where it is used — the inspector, loading a scene, a script call.
- **It does not extend to arbitrary types.** A field whose type the system does
  not understand is skipped, with a compile-time warning rather than a silent
  omission.

## Open questions

- Whether field access uses raw offsets (fast, requires `unsafe`, needs
  `#[repr(C)]` discipline) or generated accessor functions (safe, slightly
  slower, more generated code). Leaning towards generated accessors: the
  performance difference is irrelevant at inspector rates, and `unsafe` in the
  most foundational crate is a poor trade.
- How generic actor types are handled, if they are allowed at all.
- Whether components need their own reflection separate from the actors that
  hold them, or whether nesting `TypeInfo` is enough. Nesting probably suffices.
