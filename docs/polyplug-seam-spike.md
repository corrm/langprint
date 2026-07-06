# Spike: the langprint ⇄ polyplugc seam

**Slice:** ONE contract (`pipeline.Decoder::decode`), ONE language (Rust, guest side), end to end.
**Goal:** prove "langprint owns FORM, polyplugc owns LOGIC" on the narrowest real function
before any multi-language build-out. **Status: seam is clean on this slice** (byte-faithful FORM),
with three named gaps that bound how far it generalizes.

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

## 5. Gaps found (what bounds generalization)

The `decode_abi` **free function** seam is clean. The rest of the contract surface needs FORM types
langprint does not have yet:

1. **No Rust trait declaration.** The guest contract trait
   `pub trait PipelineDecoderGuestContract { fn decode(&self, input: StringView) -> Result<StringView, GuestError>; }`
   is pure FORM, but langprint's Rust backend has no `RustTrait` type and does not implement
   `InterfaceRenderer` for Rust (it exists for other backends). **Add `RustTrait` FORM before widening.**
2. **No extern block.** The author-factory forward decl
   `unsafe extern "Rust" { fn polyplug_create_decoder(host: HostContext) -> Box<dyn …>; }`
   has no langprint representation (a foreign-fn decl inside an `extern` block, distinct from a fn
   carrying `abi`). **Add an extern-block FORM item before widening.**
3. **Minor fidelity:** the real wrapper carries a free-standing `// SAFETY: …` line comment between
   doc and attribute. langprint emits only `///` docs, not bare `//` comments on an item. Fold it into
   `docs` or the LOGIC body's first line — not a blocker.

`RustStruct` already carries `methods: Vec<RustFunction>` (inherent `impl`), so the guest impl
struct + its method bodies are already expressible; the gap is specifically the **trait** and the
**extern block**.

## 6. Verdict + hand-off

- **Clean on slice #1.** The FORM/LOGIC line is real and lands exactly at `body`. Widen the seam to
  the full `Decoder` contract only after `RustTrait` (gap 1) exists; that's the next cheap increment.
- **polyplugc side goes through the polyplug agent + its gate** (`just gate`, lefthook; Rules: no
  `.unwrap()`, explicit types, no cross-crate re-exports). Do NOT edit polyplugc from the langprint
  side — two agents on the same files will collide and it won't survive the gate.
- **Hand-off deliverable:** this spec + `examples/polyplug_seam_spike.rs` (the FORM emitter for
  `decode_abi`). The polyplug agent implements the consuming half: for `decode`, build a
  `RustFunction` from the `ResolvedFunction` IR, render its FORM via langprint, and keep polyplugc's
  existing body computation as the `body: Some(Vec<String>)`. Prove one function compiles under the
  gate, then report before generalizing to the other three `Decoder` functions.
