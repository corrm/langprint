# Spike: the langprint ⇄ polyplugc seam

**Scope:** the FULL `pipeline.Decoder` contract, ONE language (Rust, guest side), end to end.
**Goal:** prove "langprint owns FORM, polyplugc owns LOGIC" and complete langprint's Rust FORM
surface before any multi-language build-out. **Status: Rust FORM is complete** — the entire guest
contract (trait + method + author-factory extern block + ABI wrapper) reproduces polyplugc's golden
output **byte-for-byte**; the three gaps found in the first slice are closed. Hand-off trigger reached.

Reproduce: `cargo run --example polyplug_seam_spike` (in this repo).

---

## 1. The contract (input)

polyplug authors contracts as TOML (`polyplug/examples/api.toml`), parsed into
`ResolvedFunction` IR by polyplugc:

```toml
[[plugin_contract]]
name = "pipeline.Decoder"
[[plugin_contract.functions]]
name = "decode"
params = [{ name = "input", type = "StringView" }]
return = "StringView"
```

## 2. The target artifact (output)

The guest-side ABI wrapper polyplugc's Rust generator hand-builds today via `out.push_str(...)`
(`polyplug/crates/polyplugc/src/generators/rust.rs:941`):

```rust
/// ABI wrapper for decode (function_id = 7).
#[allow(clippy::unnecessary_cast)]
extern "C" fn decoder_decode_abi(instance: GuestContractInstance, args: *const (), out: *mut (), out_err: *mut AbiError) {
    <~40 lines of validation / catch_unwind / arg-unpack / impl_ref.decode(..) / result-pack>
}
```

## 3. Where FORM ends and LOGIC begins

The seam runs **exactly at the function body**. langprint's `RustFunction.body: Option<Vec<String>>`
is the single field that crosses it.

| Piece | Owner | langprint representation |
|---|---|---|
| doc line `/// ABI wrapper for decode …` | **FORM** (langprint) | `docs: Some(vec![…])` |
| attribute `#[allow(clippy::unnecessary_cast)]` | **FORM** | `attributes: vec!["allow(clippy::unnecessary_cast)"]` |
| `extern "C" fn` qualifier | **FORM** | `abi: Some("C")`, `is_unsafe: false` |
| name `decoder_decode_abi` | **FORM** | `name` |
| params `instance/args/out/out_err` + their types | **FORM** | `parameters: Vec<RustParameter>` (raw type strings) |
| unit return (out-param ABI) | **FORM** | `return_type: None` |
| block scaffold: braces, indentation, per-line placement | **FORM** | rendered by langprint |
| the ~40 marshalling/dispatch lines | **LOGIC** (polyplugc) | `body: Some(vec![…])` — polyplugc computes these |

langprint verified reproducing the FORM **byte-for-byte** against the target above. With
`body: None` it emits the bare declaration terminated by `;`; with `body: Some(lines)` it opens
the block and drops each polyplugc line in verbatim, one indent deeper.

## 4. The inversion (important)

Today polyplugc owns **both** the FORM *and* the dispatch LOGIC — 6 hand-rolled string-builder
generators (`generators/{rust,cpp,csharp,python,lua,js_quickjs}.rs`, `rust.rs` alone is 5,149 lines).
The integration hypothesis this spike proves out: **polyplugc delegates FORM emission to langprint**
(build a `RustFunction`, call `render_function`) and keeps computing only the body `Vec<String>`.
That deletes the signature/scaffold half of every `out.push_str` in those generators.

## 5. Gaps — all closed

The three gaps that bounded generalization are now closed in langprint, so the **entire** guest-side
Rust FORM of a contract is expressible:

1. **Rust trait declaration** — `RustTrait` (`rust_backend::trait_types`), rendered via
   `RustBackend::render_trait`. Name + visibility + generics + supertrait bounds (`: Send + Sync`) +
   bodyless-`RustFunction` methods. A Rust trait has no target-blind IR analogue, so it is a
   backend-native render entry point (like the Python plain-`class` renderer), not an
   `InterfaceRenderer` impl.
2. **Extern block** — `RustExternBlock`, rendered via `RustBackend::render_extern_block`. Carries the
   block ABI + `unsafe` and a list of bodyless `RustFunction` items (the block owns the ABI, so items
   carry no `abi` of their own): `unsafe extern "Rust" { fn polyplug_create_decoder(…); }`.
3. **Bare `//` comments** — `RustFunction.comments: Vec<String>`, rendered between the `///` docs and
   the `#[…]` attributes, so the wrapper's `// SAFETY: …` line is now reproduced. (Lowering a
   commented function to the IR emits a dropped-comment `ConversionWarning`, per the no-silent-drop
   rule — the IR carries only `///` docs.)

`RustStruct` already carried `methods: Vec<RustFunction>` (inherent `impl`), so the guest impl struct
+ method bodies were already expressible. **Rust FORM is now complete for the contract surface.**

## 6. Acceptance — full contract, byte-identical

`examples/polyplug_seam_spike.rs` (and gate-enforced `tests/rust_trait_and_extern.rs`) reproduce the
**entire** `pipeline.Decoder` guest FORM **byte-for-byte** against polyplugc's committed golden output:

- guest trait + method → identical to `contracts.rs:13-16`
- author-factory extern block → identical to `interfaces.rs:58-63`
- `decode` ABI wrapper (incl. its `// SAFETY` comment) → identical to `rust.rs`'s emission; `body`
  is the LOGIC slot.

The `assert_eq!`s run on every `cargo run --example polyplug_seam_spike` and in CI via the tests.
This is the "Rust FORM is complete" bar — the real unit the consuming half delegates against.

## 7. Hand-off (trigger reached)

Rust FORM is complete and the full-contract diff is byte-identical, so the polyplugc-consuming half
is ready to build — **by the polyplug agent, behind polyplug's `just gate` + lefthook** (Rules: no
`.unwrap()`, explicit types, no cross-crate re-exports). Do NOT edit polyplugc from the langprint
side — two agents on the same files collide and it won't survive the gate.

- **Hand-off artifacts:** this spec + `examples/polyplug_seam_spike.rs` (the complete FORM emitter) +
  `tests/rust_trait_and_extern.rs` (the golden bars).
- **Consuming-half plan:** for each `ResolvedContract`, build a `RustTrait` + `RustExternBlock` +
  per-function `RustFunction` ABI wrappers from the polyplugc IR, render their FORM via langprint, and
  keep polyplugc's existing body computation as the `body: Some(Vec<String>)` / `comments`. Prove the
  Decoder contract's generated output is byte-identical to today's golden, then widen to the other
  five languages (each needs its own FORM-completeness pass in langprint first).
