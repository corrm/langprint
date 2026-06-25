# langprint

A multi-language source-declaration code-generation library for Rust.

langprint builds and renders **declarations** — types, fields, enums, function signatures,
visibility, namespaces, and docs — for **C++, Rust, and C#**, and can convert a declaration from
one language into another. It is the engine behind generated SDKs: it does not parse or execute
code, it emits the *shape* of an API.

## Installation

```sh
cargo add langprint
```

or add it to `Cargo.toml` directly:

```toml
[dependencies]
langprint = "0.1"
```

## Two layers

langprint deliberately has two layers, and you choose how deep you go:

1. **Native models (primary).** Each backend owns a rich, full-power model — `Cpp*`, `Rust*`,
   `CSharp*` — that expresses everything that language can say about a declaration (C++ bit-fields
   and `alignas`, Rust derives and tuple structs, C# properties and `[Flags]` enums, …). You build
   these directly and render them. Single-language generation never touches the IR and loses
   nothing.

2. **Neutral declaration IR (optional bridge).** A language-agnostic `Language*` IR models only the
   *common declaration subset*. It exists purely to move a declaration **across** languages.
   `to_ir` lowers a native model into the IR; `from_ir` raises the IR into another backend's native
   model, choosing that language's idioms (a method becomes a Rust `impl` fn, a C# instance method,
   …).

### Honest, lossy conversion

There is no universal lossless IR — that is a known impossibility, not a missing feature. langprint
does not pretend otherwise. Instead the IR is **scoped** to what genuinely crosses all three
languages, and every feature that cannot cross is **reported, never silently dropped**:

- A Rust data-carrying enum variant → C++ enum: the payload is dropped and a
  `ConversionWarning::UnsupportedFeature` is emitted (a C++ enum holds no per-variant data).
- A C# class with a base and an interface → Rust: both are reported as dropped (Rust has no
  inheritance), while the fields cross cleanly.

You always know exactly what a conversion gave up.

## Quick start

```rust
use langprint::backends::BackendItem;
use langprint::backends::csharp_backend::{CSharpBackend, CSharpType};
use langprint::backends::rust_backend::{RustBackend, RustField, RustStruct, RustVisibility};
use langprint::renderers::StructRenderer;

let player = RustStruct {
    name: "Player".to_string(),
    visibility: RustVisibility::Pub,
    generic_args: vec![],
    fields: vec![RustField {
        name: "health".to_string(),
        field_type: "f32".to_string(),
        visibility: RustVisibility::Pub,
        attributes: vec![],
        docs: None,
    }],
    methods: vec![],
    derives: vec![],
    attributes: vec![],
    is_tuple: false,
    docs: None,
};

// Layer 1 — render the native model directly.
let rust_src = RustBackend::default()
    .render_struct(&player, None::<&str>, None::<&str>, None, &mut 0)
    .unwrap();

// Layer 2 — convert the declaration into C# through the neutral IR.
let ir = player.to_ir(None);
let csharp = CSharpType::from_ir(ir.value, None); // inspect csharp.log.warnings for any losses
let csharp_src = CSharpBackend::default()
    .render_struct::<&str>(&csharp.value, None, None, None, &mut 0)
    .unwrap();
```

The Rust struct renders as:

```rust
pub struct Player {
    pub health: f32,
}
```

and the same declaration, converted into C#, renders as:

```csharp
public struct Player
{
    public float Health;
}
```

The Rust `f32` is re-spelled to C# `float` and the field is PascalCased — see *Customizing
conversion* below for how both are controlled.

A complete, runnable version of this flow lives in
[`examples/cross_language.rs`](examples/cross_language.rs):

```sh
cargo run --example cross_language
```

## Customizing conversion

Cross-language `from_ir` is driven by a `ConversionConfig { type_map, rename }`, carried on each
backend's conversion options (defaulting to the built-in map with renaming on).

**TypeMap** re-spells primitive types across languages. The built-in table covers the common
primitives (`f32`↔`float`, `uint8_t`↔`u8`↔`byte`, `i32`↔`int`↔`int32_t`, …); a type it does not
recognize is emitted verbatim **and** reported with a `ConversionWarning`. You can override, extend,
or clear it:

```rust
use langprint::{ConversionConfig, PrimitiveType, TargetLanguage, TypeMap};

let mut type_map = TypeMap::builtin();
type_map.insert_spelling("FString", PrimitiveType::Str);          // recognize a game type
type_map.set_output(PrimitiveType::Str, TargetLanguage::CSharp, "string"); // override output
// type_map.clear();                                              // start from nothing

let config = ConversionConfig::new(type_map, /* rename = */ false);
```

**Renaming.** With `rename` on (the default), `from_ir` rewrites identifiers to the target
language's convention (Rust `snake_case` fns/fields; C# `PascalCase` types/methods/fields/enum
members; C++ left verbatim) and reports each change as `ConversionWarning::NamingConventionChanged`.
Set `rename: false` to keep identifiers exactly as written.

## Namespaces

Namespaces/modules are first-class and render across every backend — C++ `namespace X { … }`,
Rust `mod x { … }`, C# `namespace X { … }` — nesting their defines, constants, enums, structs, free
functions, and child namespaces via the same per-member renderers. Cross-language conversion threads
the `ConversionConfig` into every member, so type mapping and renaming apply throughout (e.g. a Rust
`mod` name is snake_cased, a C# namespace PascalCased). Where a target cannot express a member — C#
has no namespace-level free functions — it is dropped with a `ConversionWarning`, never silently.

## Backends

| Language | Native model prefix | Notable features modelled |
| -------- | ------------------- | ------------------------- |
| C++      | `Cpp*`              | structs/classes/unions, bit-fields, `alignas`, enum classes, templates |
| Rust     | `Rust*`             | structs + inherent `impl` blocks, derives, tuple structs, enums with data |
| C#       | `CSharp*`           | classes/structs/records, properties, interfaces, `[Flags]` enums, sealing rules |

`langprint::AVAILABLE_BACKENDS` is the live list.

## Project generators

Beyond single declarations, langprint can emit the surrounding build project for a generated SDK
via `langprint::project_gen`:

- `CmakeGenerator`, `MakefileGenerator` (C/C++)
- `VslnGenerator` / `SlnxGenerator` (Visual Studio solutions)
- `CargoGenerator` (Rust)
- `CSharpProjectGenerator` (.NET SDK-style `.csproj`)

### Convenience helpers

For the common workflow of rendering declarations to files then generating build files, two helpers
eliminate boilerplate:

```rust
use langprint::project_gen::{ProjectSpec, write_files, OutputKind, LanguageStandard};
use std::path::PathBuf;

// Render your declarations to strings, collect as (path, content) pairs.
let files: Vec<(PathBuf, String)> = /* ... */;

// Write rendered files to disk (creates parent dirs).
write_files(&files, &output_dir)?;

// Build a spec, auto-classifying sources/headers from the file list.
let spec = ProjectSpec::new("my_project", LanguageStandard::Cpp17, OutputKind::StaticLib)
    .populate_from_files(&files);

// Generate the build files.
CmakeGenerator.generate(&spec, &output_dir)?;
```

`populate_from_files` classifies `.h`/`.hpp`/`.hxx` as headers, everything else as sources, and
infers `include_dirs` from parent directories. Both helpers are optional — you retain full control
over rendering and spec construction.
### ProjectBuilder

For fluent, chainable spec construction, use `ProjectBuilder`:

```rust
use langprint::project_gen::{ProjectBuilder, LanguageStandard, OutputKind, Platform, Arch};

let spec = ProjectBuilder::new("my_lib", LanguageStandard::Cpp17, OutputKind::StaticLib)
    .sources(["src/main.cpp", "src/types.cpp"])
    .headers(["include/types.h"])
    .include_dirs(["include"])
    .define("DEBUG", Some("1"))
    .platform(Platform::Linux)
    .build()
    .unwrap();
```

The builder supports all `ProjectSpec` fields across every language (C, C++, C#, Rust). It also
carries `populate_from_files` so you can chain file classification directly:

```rust
let spec = ProjectBuilder::new("my_lib", LanguageStandard::Rust2021, OutputKind::SharedLib)
    .populate_from_files(&files)
    .build()
    .unwrap();
```

`build()` validates the spec (non-empty name, at least one source file, consistent PCH config)
and returns `Result<ProjectSpec, ProjectGenError>`.
## Scope

langprint models declarations and their layout, not arbitrary source code or runtime behavior. If
you need a feature only one language has, use that language's native model — it is the primary API
and never the lossy one. The neutral IR is only for crossing languages, and it tells you what it
could not carry.

## License

Licensed under the MIT license ([LICENSE-MIT](LICENSE-MIT)).
