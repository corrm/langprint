# langprint

A multi-language source-declaration code-generation library for Rust.

langprint builds and renders **declarations** — types, fields, enums, function signatures,
visibility, namespaces, and docs — for **C++, Rust, C#, Python, Lua, and JavaScript**, and can
convert a declaration from one language into another. It is the engine behind generated SDKs: it
does not parse or execute code, it emits the *shape* of an API.

The three typed languages (C++, Rust, C#) have rich, full-power native models and participate in
conversion both ways. The three near-untyped languages (Python, Lua, JS) are deliberately thin and
are **render targets only** — you lower the IR *to* them; you never transpile *from* them. No fake
type system is bolted onto a language that does not have one.

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

You always know exactly what a conversion gave up. Attributes are **no longer dropped wholesale** —
see *Annotations* below.

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

let mut type_map = TypeMap::default();
type_map.insert_spelling("FString", PrimitiveType::Str);          // recognize a game type
type_map.set_output(PrimitiveType::Str, TargetLanguage::CSharp, "string"); // override output
// type_map.clear();                                              // start from nothing

let config = ConversionConfig::new(type_map, /* rename = */ false);
```

**Python ctypes** uses the same [`TypeMap`](#typemap) as every other backend. The
`ctypes_type_map()` function under `python_backend` returns a ready-to-use `TypeMap` with ctypes
spellings for Python output:

```rust
use langprint::{ConversionConfig, PrimitiveType, TargetLanguage, TypeMap};
use langprint::backends::python_backend::ctypes_type_map;

let config = ConversionConfig::new(ctypes_type_map(), false);
```

Custom types (e.g. `MyHandle`→`ctypes.c_void_p`) go through `type_override` on `ConversionConfig`.
Types not covered by your `TypeMap` pass through verbatim with an `UnsupportedFeature` warning —
provide a mapping to suppress it.

**Renaming.** With `rename` on (the default), `from_ir` rewrites identifiers to the target
language's convention (Rust `snake_case` fns/fields; C# `PascalCase` types/methods/fields/enum
members; C++ left verbatim) and reports each change as `ConversionWarning::NamingConventionChanged`.
Set `rename: false` to keep identifiers exactly as written.

**NamingMap** drives those conventions as a clone-and-override table like `TypeMap`, keyed by
`(TargetLanguage, IdentifierKind)`. Override a single entry — e.g. make Python functions
`PascalCase`:

```rust
use langprint::{CaseStyle, ConversionConfig, NamingMap, TargetLanguage};
use langprint::convert::IdentifierKind;

let mut naming_map = NamingMap::default();
naming_map.insert(TargetLanguage::Python, IdentifierKind::Function, CaseStyle::Pascal);
let config = ConversionConfig { naming_map, ..ConversionConfig::default() };
```

**KeywordMap** escapes identifiers that collide with a target reserved word (Rust `r#ident`, C#
`@ident`, others `ident_`) — applied even with `rename` off, since a collision is a correctness
issue. A field named `class` becomes `class_` in Python; extend the set for your own reserved words:

```rust
use langprint::{ConversionConfig, KeywordMap, TargetLanguage};

let mut keyword_map = KeywordMap::default();
keyword_map.insert(TargetLanguage::Python, "mykw");   // also escape `mykw` → `mykw_`
let config = ConversionConfig { keyword_map, ..ConversionConfig::default() };
```

**AnnotationMap** controls the native spelling a Tier-1 `Annotation` lowers to per language, for
the textual backends (Rust, C#). It is a clone-and-override table like `TypeMap`, keyed by
`(TargetLanguage, AnnotationKind)`; the template's `{n}` placeholder is filled with the alignment
value for `Aligned`. A `(language, kind)` with no entry emits nothing (this is how C# `Aligned`
stays absent). C++ alignment is numeric (`alignas(n)`) and is not part of this textual map.

```rust
use langprint::{AnnotationKind, AnnotationMap, ConversionConfig, TargetLanguage};

let mut annotation_map = AnnotationMap::default();
annotation_map.insert(TargetLanguage::Rust, AnnotationKind::ReprC, "repr(C, packed)"); // override
annotation_map.insert(TargetLanguage::CSharp, AnnotationKind::Aligned, "StructLayout(LayoutKind.Sequential, Size = {n})"); // add
let config = ConversionConfig { annotation_map, ..ConversionConfig::default() };
```

## Namespaces

Namespaces/modules are first-class and render across every backend — C++ `namespace X { … }`,
Rust `mod x { … }`, C# `namespace X { … }` — nesting their defines, constants, enums, structs, free
functions, and child namespaces via the same per-member renderers. Cross-language conversion threads
the `ConversionConfig` into every member, so type mapping and renaming apply throughout (e.g. a Rust
`mod` name is snake_cased, a C# namespace PascalCased). Where a target cannot express a member — C#
has no namespace-level free functions — it is dropped with a `ConversionWarning`, never silently.

## Backends

| Language | Native model prefix | Role | Notable features modelled |
| -------- | ------------------- | ---- | ------------------------- |
| C++      | `Cpp*`              | typed (to/from IR) | structs/classes/unions, bit-fields, `alignas`, enum classes, templates, `extern "C"` |
| Rust     | `Rust*`             | typed (to/from IR) | structs + inherent `impl` blocks, derives, tuple structs, enums with data, `unsafe`, `extern "C"` ABI |
| C#       | `CSharp*`           | typed (to/from IR) | classes/structs/records, properties, interfaces, `[Flags]` enums, sealing rules, `unsafe` modifier (methods/classes, never structs) |
| Python   | `Python*`           | thin, render target (from IR only) | `ctypes.Structure`, `enum.IntEnum`, `class`/`def` with body slot, PEP-484 hints where real |
| Lua      | `Lua*`              | thin, render target (from IR only) | module tables (`local M = {}` … `return M`), functions, field assignment; no types/visibility |
| JS       | `Js*`               | thin, render target (from IR only) | `class`/`extends`, functions, fields, optional JSDoc (signatures stay untyped) |

`langprint::AVAILABLE_BACKENDS` is the live list.

### Native FFI qualifiers

Declaration-level FFI qualifiers are modelled natively, not via hooks: Rust `abi: Option<String>`
renders `pub unsafe extern "C" fn …`; C++ `is_extern_c` renders the `extern "C"` linkage specifier;
C# `is_unsafe` renders the `unsafe` modifier on methods and classes. C# **structs are kept safe by
construction** — `CSharpTypeKind::can_be_unsafe()` returns `false` for `Struct`, so a struct can
never render `unsafe`.

## Body-slot contract

langprint emits source *declarations*. A function/method is a signature plus a body slot the
consumer fills with raw strings — langprint never models statements or expressions. Every
backend's function type carries `body: Option<Vec<String>>`:

- `body: None` → a bare declaration terminated for the language (`;`).
- `body: Some(lines)` → the signature, an open block, each `line` emitted **verbatim** one indent
  deeper, then the close block. langprint adds only indentation and block punctuation.

```rust
RustFunction { name: "add".into(), body: Some(vec!["a + b".into()]), /* … */ }
// pub fn add(a: i32, b: i32) -> i32 {
//     a + b
// }
```

C++ gates the body slot behind its `render_definition` render option (a header normally wants
declarations only); set `render_definition: true` to emit the block. The contract is locked by
`tests/body_slot_contract.rs`.

## Annotations

Native attributes, derives, `repr`, and layout no longer vanish when a declaration crosses the IR.
They are preserved in two tiers (`langprint::ir::{Annotation, RawAttribute}`):

- **Tier 1 — curated layout vocabulary.** A small, closed `Annotation` enum of source-neutral
  facts — `ReprC`, `Packed`, `Aligned(n)` — that **translates** across languages. A concept is
  admitted only when at least two backends each map it to native syntax, so Rust `#[repr(C)]`
  becomes C# `[StructLayout(LayoutKind.Sequential)]`, and Rust `#[repr(align(8))]` becomes C++
  `alignas(8)`. The IR stays target-blind: a variant names a fact, not a target's spelling.
- **Tier 2 — opaque carry.** Everything else (`derive(Clone)`, `[DllImport]`, …) is carried as a
  `RawAttribute { source, text }`: `text` is always the inner value, never pre-wrapped syntax.
  `render_raw_attributes(language, site, values)` owns delimiters and emits each entry as its own
  line. For example, Rust `repr(u32)` becomes `#[repr(u32)]`, C# `Flags` becomes `[Flags]`, and
  a C# return value becomes `[return: Flags]`.

  Rust root attributes are rendered as inner attributes (`#![value]`); all other Rust declaration
  sites, including returns, use ordinary outer attributes on the function declaration
  (`#[must_use]`). C++ uses standard `[[value]]` attributes where grammar permits; return values
  use that same grammar-valid function-prefix position. Root metadata is
  `// [[langprint::root(value)]]` because C++ has no translation-unit attachment point. C# uses
  `[assembly: value]`, `[module: value]`, and `[return: value]` for its three targeted sites.
  Python can attach native decorators only to types, enums, and functions; root, module, field,
  variant, parameter, and return values use `# @langprint Site: value` metadata. Lua uses
  `---@langprint Site: value`, and JavaScript/TypeScript uses
  `/** @langprint Site: value */`. These metadata forms retain the exact node identity at sites
  whose language grammar has no native attribute position.

## Imports

Each backend can track and render its own imports — deduplicated and deterministically ordered — in
native syntax: C++ `#include`, C# `using`, Rust `use`, Python `import`/`from … import`, Lua
`require`, JS/TS `import`. An `ImportSet` collects entries; an `ImportMap` (built-in + extensible,
like `TypeMap`) resolves a referenced type to its import so it appears automatically:

```rust
use langprint::{ImportMap, ImportSet, TargetLanguage};

let map = ImportMap::builtin(TargetLanguage::Cpp);
let mut imports = ImportSet::new(TargetLanguage::Cpp);
imports.add_type_ref("uint32_t", &map); // -> #include <cstdint>
let header = imports.render();
```

Import rendering is additive: a backend that registers nothing renders exactly as before.

### `ImportEntry` variants

Each entry carries exactly what one language's syntax needs. `render()` switches on the target
language and reads only the variants it uses:

| Variant | Language | Renders |
|---|---|---|
| `Include { header, system }` | C++ | `#include <h>` / `#include "h"` |
| `Using(ns)` | C# | `using ns;` |
| `Use(path)` | Rust | `use path;` |
| `PyImport(module)` | Python | `import module` |
| `PyFrom { module, symbol }` | Python | `from module import symbol` |
| `Require { name, module }` | Lua | `local name = require("module")` |
| `JsDefault { name, source }` | JS/TS | `import name from 'source'` |
| `JsNamed { name, source }` | JS/TS | `import { name } from 'source'` |
| `JsTypeNamed { name, source }` | TS | `import type { name } from 'source'` |
| `JsTypeNamespace { alias, source }` | TS | `import type * as alias from 'source'` |
| `JsReexport { name, source }` | JS/TS | `export { name } from 'source'` |

### Grouping and ordering

`render()` emits each language's idiomatic order, and — crucially — **merges entries that share a
target** onto a single line:

- **Python** — `from __future__ import …` is emitted first (Python requires it ahead of every
  statement), then `import x` lines, then `from x import …` lines. Multiple `PyFrom` entries with the
  same module collapse to one `from module import a, b, c` (symbols sorted).
- **JS/TS** — a fixed kind order (default, named, type-named, `type * as`, re-export). Multiple
  `JsNamed` / `JsTypeNamed` / `JsReexport` entries sharing a `source` collapse to one
  `import { a, b } from 'source'` (or `import type { … }` / `export { … } from`), names sorted.
- **C++** — system `<…>` includes before local `"…"`, alphabetical within each.
- **C# / Rust / Lua** — alphabetical by namespace / path / binding name.

```rust
let mut ts = ImportSet::new(TargetLanguage::Js);
ts.add(ImportEntry::JsNamed { name: "b".into(), source: "./m".into() });
ts.add(ImportEntry::JsNamed { name: "a".into(), source: "./m".into() });
ts.add(ImportEntry::JsTypeNamed { name: "T".into(), source: "./m".into() });
// import { a, b } from './m';
// import type { T } from './m';
assert_eq!(ts.render(), "import { a, b } from './m';\nimport type { T } from './m';\n");
```

## Extension hooks

Single-language native generation is lossless and needs no hooks. For the cross-language IR path,
`ConversionConfig` carries two **opt-in, no-op-by-default** extension points (`langprint::convert`):

- `type_override` — a closure consulted before the `TypeMap` for custom type resolution.
- `hooks: Option<Arc<dyn ConversionHooks>>` — `after_to_ir_*` / `before_from_ir_*` callbacks (struct,
  function, enum) to re-apply or remap what the IR cannot carry.

Separately, `renderers::post_process` is a post-render utility **function** (not a `ConversionConfig`
field): pass it rendered output to wrap or prepend a preamble (e.g. a `#pragma once`).

## Project generators

Beyond single declarations, langprint can emit the surrounding build project for a generated SDK
via `langprint::project_gen`:

- `CmakeGenerator`, `MakefileGenerator` (C/C++)
- `VslnGenerator` / `SlnxGenerator` (Visual Studio solutions)
- `CargoGenerator` (Rust)
- `CSharpProjectGenerator` (.NET SDK-style `.csproj`)

### ProjectBuilder

Construct a spec with the fluent builder:

```rust
use langprint::project_gen::{ProjectBuilder, LanguageStandard, OutputKind, Platform};

let spec = ProjectBuilder::new("my_lib", LanguageStandard::Cpp17, OutputKind::StaticLib)
    .sources(["src/main.cpp", "src/types.cpp"])
    .headers(["include/types.h"])
    .include_dirs(["include"])
    .define("DEBUG", Some("1"))
    .platform(Platform::Linux)
    .build()
    .unwrap();
```

It works across all supported languages:

```rust
// Rust crate
let spec = ProjectBuilder::new("my_crate", LanguageStandard::Rust2021, OutputKind::SharedLib)
    .source("src/lib.rs")
    .build()
    .unwrap();

// C# project
let spec = ProjectBuilder::new("MyLib", LanguageStandard::CSharp12, OutputKind::SharedLib)
    .source("Program.cs")
    .build()
    .unwrap();
```

Chain `populate_from_files` to auto-classify sources/headers from rendered output:

```rust
let files: Vec<(PathBuf, String)> = /* rendered declarations */;

let spec = ProjectBuilder::new("my_lib", LanguageStandard::Cpp17, OutputKind::StaticLib)
    .populate_from_files(&files)
    .build()
    .unwrap();
```

This classifies `.h`/`.hpp`/`.hxx` as headers, everything else as sources, and infers
`include_dirs` from parent directories. `write_files` is available for disk I/O.

`build()` validates the spec (non-empty name, at least one source file, consistent PCH config)
and returns `Result<ProjectSpec, ProjectGenError>`.
## Scope

langprint models declarations and their layout, not arbitrary source code or runtime behavior. If
you need a feature only one language has, use that language's native model — it is the primary API
and never the lossy one. The neutral IR is only for crossing languages, and it tells you what it
could not carry.

### Known scope boundaries

The neutral IR carries opaque attribute lists on modules, types, fields, enums, variants,
functions, parameters, and returns. Field initializers remain outside the neutral IR.

## License

Licensed under the MIT license ([LICENSE-MIT](LICENSE-MIT)).
