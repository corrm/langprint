//! Spike: **langprint owns FORM, polyplugc owns LOGIC** — proven on ONE contract, ONE
//! language, end to end. Not a build-out; the point is to show the *shape* of the seam.
//!
//! Contract (polyplug `examples/api.toml`):
//! ```toml
//! [[plugin_contract]]
//! name = "pipeline.Decoder"
//! [[plugin_contract.functions]]
//! name = "decode"
//! params = [{ name = "input", type = "StringView" }]
//! return = "StringView"
//! ```
//!
//! Target artifact: the guest-side ABI wrapper `decoder_decode_abi` that polyplugc's Rust
//! generator hand-builds today via `out.push_str(...)`
//! (polyplug `crates/polyplugc/src/generators/rust.rs:941`). This spike shows langprint
//! emitting that function's **FORM** — the `extern "C"` signature, the `#[allow]` attribute,
//! the doc line, and the block scaffold — while polyplugc keeps ownership of the **LOGIC**:
//! the marshalling body lines, handed across the seam as `body: Some(Vec<String>)`.
//!
//! Run: `cargo run --example polyplug_seam_spike`

use langprint::backends::rust_backend::{
    RustBackend, RustFunction, RustParameter, RustSelfKind, RustVisibility,
};
use langprint::renderers::FunctionRenderer;

/// The FORM of the `decode` ABI wrapper. Everything here is owned by langprint; `body` is the
/// single seam field polyplugc fills. `body: None` => bare declaration; `Some(lines)` => block.
fn decode_abi_form(body: Option<Vec<String>>) -> RustFunction {
    RustFunction {
        name: "decoder_decode_abi".to_string(),
        // The wrapper is referenced by function pointer in a static table, not exported by name.
        visibility: RustVisibility::Private,
        self_kind: RustSelfKind::None,
        parameters: vec![
            RustParameter {
                name: "instance".to_string(),
                param_type: "GuestContractInstance".to_string(),
            },
            RustParameter {
                name: "args".to_string(),
                param_type: "*const ()".to_string(),
            },
            RustParameter {
                name: "out".to_string(),
                param_type: "*mut ()".to_string(),
            },
            RustParameter {
                name: "out_err".to_string(),
                param_type: "*mut AbiError".to_string(),
            },
        ],
        generic_args: Vec::new(),
        // Out-param ABI: the result is written through `out` / `out_err`, so the fn returns `()`.
        return_type: None,
        is_unsafe: false,
        is_async: false,
        is_const: false,
        abi: Some("C".to_string()),
        body,
        attributes: vec!["allow(clippy::unnecessary_cast)".to_string()],
        docs: Some(vec![
            "ABI wrapper for decode (function_id = 7).".to_string(),
        ]),
    }
}

fn main() {
    let rust = RustBackend::default();

    // ---- FORM only (body: None): langprint emits the declaration, LOGIC slot empty. ----
    let declaration = rust
        .render_function(
            &decode_abi_form(None),
            None::<&str>,
            None::<&str>,
            None,
            &mut 0,
        )
        .expect("render declaration");
    println!("// ===== FORM only (body: None) — the seam, no LOGIC =====");
    println!("{declaration}\n");

    // ---- FORM + LOGIC (body: Some): polyplugc supplies the marshalling lines. ----
    // These lines are OWNED BY polyplugc — this is a stand-in for what its generator computes.
    // langprint owns only their punctuation and indentation inside the block.
    let logic: Vec<String> = vec![
        "// --- LOGIC owned by polyplugc: validation, dispatch, marshalling ---".to_string(),
        "let __result_err: AbiError = (|| {".to_string(),
        "    if instance.data.is_null() {".to_string(),
        "        return AbiError { code: AbiErrorCode::InvalidPointer as u32, \
            message: string_view_from_static(b\"instance is null\") };"
            .to_string(),
        "    }".to_string(),
        "    let state: &DecoderState = unsafe { &*(instance.data as *const DecoderState) };"
            .to_string(),
        "    let impl_ref: &dyn PipelineDecoderGuestContract = state.implementation.as_ref();"
            .to_string(),
        "    // unpack args -> input: StringView, call impl_ref.decode(input), write into `out`"
            .to_string(),
        "    AbiError { code: 0, message: string_view_from_static(b\"\") }".to_string(),
        "})();".to_string(),
        "unsafe { core::ptr::write(out_err, __result_err); }".to_string(),
    ];
    let definition = rust
        .render_function(
            &decode_abi_form(Some(logic)),
            None::<&str>,
            None::<&str>,
            None,
            &mut 0,
        )
        .expect("render definition");
    println!("// ===== FORM + LOGIC (body: Some) — langprint scaffold, polyplugc body =====");
    println!("{definition}");
}
