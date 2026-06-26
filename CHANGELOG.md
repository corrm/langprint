# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-06-26

The emitter release: langprint becomes a complete source-declaration emitter for six
backends (C++, Rust, C#, Python, Lua, JavaScript) with a target-blind conversion IR.

### Added
- **Three thin, untyped backends** — Python (ctypes `Structure`, `IntEnum`, `class`/`def`),
  Lua (modules/functions), and JavaScript (classes/functions). They render only FORM and
  never fabricate a type system.
- **Body-slot contract** — every function carries `body: Option<Vec<String>>`: `None` emits a
  bare declaration, `Some(lines)` emits a block with the consumer's verbatim lines. langprint
  owns punctuation and indentation, never statements.
- **Two-tier IR annotation system** — Tier-1 `Annotation` is a curated, source-neutral layout
  vocabulary (`ReprC`, `Packed`, `Aligned`); Tier-2 `RawAttribute` carries opaque source-tagged
  attributes that round-trip losslessly within their own language. The IR no longer blanket-drops
  native attributes/derives/repr.
- **Native FFI / unsafe qualifiers** — Rust `extern "C"`/`unsafe`, C++ `extern "C"`, C# `unsafe`.
- **Per-backend imports** — `ImportMap`/`ImportSet` for `#include`/`use`/`using`/`require`.
- **Opt-in conversion hooks** — `ConversionHooks` (`after_to_ir_*`/`before_from_ir_*`),
  `type_override`, and `renderers::post_process`; all no-ops by default.
- **Configurable mapping tables** — `NamingMap` (case conventions), `KeywordMap` (reserved-word
  escaping: Rust `r#x`, C# `@x`, others `x_`), and `AnnotationMap` (annotation→native lowering),
  all following the `Default` + clone + insert/extend/clear/resolve pattern.
- **`ctypes_type_map()`** — a `TypeMap` with Python ctypes spellings; custom types via `type_override`.
- **`TypeMap::clear_output`** — remove a single `(primitive, language)` output so it surfaces as
  an unmapped-type warning.
- **C++ packed structs** — `CppStruct.is_packed` renders `#pragma pack(push, 1)`/`pop` and
  round-trips through `Annotation::Packed`.
- **`ProjectBuilder`** — fluent `ProjectSpec` construction, plus `populate_from_files`/`write_files`.

### Changed
- **BREAKING** — `TypeMap`, `NamingMap`, `KeywordMap`, and `AnnotationMap` no longer expose
  `builtin()`; use `Default::default()` for the populated table and `empty()` for an empty one.
- **BREAKING** — `EnumRenderer` no longer has an `EnumVariantRenderOptions` associated type or a
  `variant_options` parameter; variant render options are nested in `RenderOptions.variant`.
  `render_enum`/`render_enum_to` now take one fewer argument.
- **BREAKING** — the Python `CtypeMap` struct is replaced by the `ctypes_type_map()` function.
- **BREAKING** — Python `PythonEnumMemberRenderOptions` removed (it had no render knobs and existed
  only to satisfy the old trait associated type).
- **BREAKING** — in the ctypes map, `i128`/`u128` no longer silently resolve to `int`; they now
  produce an unmapped-type warning so consumers supply a `type_override` (`void` still maps to `None`).

### Fixed
- C++ silently dropped `Annotation::Packed` in both conversion directions (layout/ABI data loss).
- Reserved-keyword escaping was skipped for Rust/C++ type, enum, variant, field, and function names
  on `from_ir`, producing uncompilable output on a keyword collision.
- `ctypes_type_map()` mapped `i128`/`u128` to an invalid ctypes `int` with no warning.
- Four broken `builtin` intra-doc links repointed to `default`; truncated `AnnotationMap` doc completed.
- Python multi-line docstring continuation lines were not indented.
- Lua module fields were dropped instead of round-tripping through `LanguageNamespace.constants`.
- C++ no longer fabricates a body stub for `body: None` or emits `template<…> extern "C"`.

[0.2.0]: https://github.com/corrm/langprint/compare/v0.1.1...v0.2.0
