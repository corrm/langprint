# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.3] - 2026-07-08

The import-model release: extends `ImportSet` so a consumer can express the
import shapes real generators emit — TypeScript type-only imports, namespace
imports, and re-exports — and so multi-symbol imports from one target collapse
onto a single line. Every change is additive and preserves prior single-symbol
output, so 0.2.2 consumers are unaffected.

### Added
- **`ImportEntry::JsTypeNamed { name, source }`** — a TypeScript type-only named
  import, `import type { name } from 'source'`. Groups by `source` like `JsNamed`.
- **`ImportEntry::JsTypeNamespace { alias, source }`** — a TypeScript type-only
  namespace import, `import type * as alias from 'source'`.
- **`ImportEntry::JsReexport { name, source }`** — a named re-export,
  `export { name } from 'source'`. Groups by `source`.

### Changed
- **Python `render()` puts `from __future__ import …` first**, ahead of every
  `import`, as Python requires; its symbols merge onto one line.
- **Multi-symbol merge** — multiple `PyFrom` entries sharing a module now render
  as one `from module import a, b, c` (symbols sorted) instead of one line each;
  the same merge applies to JS `JsNamed` / `JsTypeNamed` / `JsReexport` entries
  that share a `source`. Single-symbol inputs render exactly as in 0.2.2.

## [0.2.2] - 2026-07-07

The ABI-mirror release: the byte-identity fixes and one new base-class field that
let polyplug's second codegen pipeline (the `sdks/*/abi` mirrors) render its C++,
C#, and Python declaration FORM through langprint. Every change is additive and
defaults to preserving prior output, so 0.2.1 consumers are unaffected.

### Added
- **`PythonStruct.base_class`** — select `ctypes.Structure` (default) or `ctypes.Union`
  for the emitted base, so a `ctypes.Union` reuses the identical `_fields_` FORM
  (mirrors `PythonEnum.base_class`).
- **`PythonBackend.docstring_close_on_own_line`** (default `false`) — write a multi-line
  docstring's closing `"""` on its own PEP 257 indented line instead of appended to the
  last content line. Single-line docstrings are unaffected.
- **`PythonBackend.docstring_raw_on_backslash`** (default `false`) — emit `r"""…"""` when
  the docstring contains a backslash, avoiding an import-time `SyntaxWarning`.

### Fixed
- **C++ blank doc line** — a blank `///`/`//` doc line renders as the bare marker with no
  trailing space.
- **C++ struct closing brace** — no spurious blank line before the closing `};` of a
  field-only struct.
- **C# unsafe struct** — `can_be_unsafe()` allows `unsafe struct` (needed for `fixed`
  buffers); a blank C# doc line renders with no trailing space. Safe callers are unaffected.

## [0.2.1] - 2026-07-07

The FORM-seam release: completes the declaration surface polyplugc drives to emit
byte-identical guest/host bindings across all six backends. Every addition is
additive and defaults to preserving prior output, so 0.2.0 consumers are unaffected.

### Added
- **Rust FORM completion** — `RustTrait`, extern-block, and function `comments`
  rendering, so a consumer can drive full trait + `extern "C"` + ABI-wrapper form.
- **`verbatim_body` render option** on the C++, C#, Python, Lua, and JS function/method
  renderers (default `false`). When `true`, body lines are emitted exactly as given with
  no re-indentation — the seam for formatter-less languages that must reproduce
  hand-baked body whitespace byte-for-byte.
- **JS class-member methods** — `JsBackend::render_method_to` renders `[static] name(params): ret { … }`
  (no `function` keyword) for placement inside a hand-emitted `class { … }` body.
- **JavaScript TypeScript mode** — `JsFunctionRenderOptions.typescript` (and the class
  option) emit inline `param: T` / `): R` annotations from the existing
  `type_doc`/`return_type` fields; default off leaves plain JS + JSDoc unchanged.
- **Constant-table enum renderers** — `JsEnum` (`export const N = Object.freeze({…} as const)`
  plus companion type) and `LuaEnum` (`local N = {…}`), each with an `EnumRenderer` impl.
- **`PythonEnum.base_class`** — select `enum.IntEnum` / `enum.IntFlag` for the emitted base.
- **Render toggles that default to prior output** — `attributes_before_derives`
  (Rust `#[repr]` before `#[derive]`), `CSharpBackend.open_brace_on_new_line`
  (default `true` = Allman), and `CppBackend.space_before_enum_base` (`enum class N : U`).
- **`static` / `inline` on C++ free-function definitions** — previously emitted on
  declarations only.

### Fixed
- **C++ virtual methods** — emit the real `virtual` keyword instead of a `/* virtual */`
  comment, so pure-virtual `= 0` declarations compile. Added `impl Default for CppBackend`.

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
