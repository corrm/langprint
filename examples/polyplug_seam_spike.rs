//! Spike: **langprint owns FORM, polyplugc owns LOGIC**, now widened to a FULL contract.
//!
//! langprint reproduces the ENTIRE guest-side Rust FORM of the `pipeline.Decoder` contract —
//! the guest trait, its method, AND the author-factory `extern` block — **byte-identical** to
//! polyplugc's committed golden output
//! (`polyplug/examples/guests/rust/decoder/generated/guest/{contracts,interfaces}.rs`). It also
//! emits the `decode` ABI wrapper, where the seam shows: FORM (signature + `extern "C"` + attrs +
//! `//` comment + block scaffold) is langprint's; the body lines are polyplugc's, handed across as
//! `body: Some(Vec<String>)`.
//!
//! The `assert_eq!`s make `cargo run --example polyplug_seam_spike` a live acceptance check; the
//! same goldens are gate-enforced in `tests/rust_trait_and_extern.rs`.

use langprint::backends::rust_backend::{
    RustBackend, RustExternBlock, RustFunction, RustParameter, RustSelfKind, RustTrait,
    RustVisibility,
};
use langprint::renderers::FunctionRenderer;

/// Byte target: `contracts.rs:13-16`.
const GOLDEN_TRAIT: &str = "\
/// Guest trait for contract `pipeline.Decoder` (id=0xE1D7DE773BE6E7F7)
pub trait PipelineDecoderGuestContract: Send + Sync {
    fn decode(&self, input: StringView) -> Result<StringView, GuestError>;
}
";

/// Byte target: `interfaces.rs:58-63`.
const GOLDEN_EXTERN: &str = "\
unsafe extern \"Rust\" {
    /// Author-provided factory — define it in the plugin crate as:
    /// `#[unsafe(no_mangle)]`
    /// `pub fn polyplug_create_decoder(host: HostContext) -> Box<dyn PipelineDecoderGuestContract> { ... }`
    fn polyplug_create_decoder(host: HostContext) -> Box<dyn PipelineDecoderGuestContract>;
}
";

fn param(name: &str, ty: &str) -> RustParameter {
    RustParameter {
        attributes: Vec::new(),
        name: name.to_string(),
        param_type: ty.to_string(),
    }
}

/// A bodyless signature (`body: None` => `fn …;`) with the given docs — the shape used inside
/// both the trait and the extern block.
fn signature(
    name: &str,
    self_kind: RustSelfKind,
    parameters: Vec<RustParameter>,
    return_type: Option<&str>,
    docs: Option<Vec<String>>,
) -> RustFunction {
    RustFunction {
        return_attributes: Vec::new(),
        name: name.to_string(),
        visibility: RustVisibility::Private,
        self_kind,
        parameters,
        generic_args: Vec::new(),
        return_type: return_type.map(str::to_string),
        is_unsafe: false,
        is_async: false,
        is_const: false,
        abi: None,
        body: None,
        attributes: Vec::new(),
        docs,
        comments: Vec::new(),
    }
}

fn decoder_trait() -> RustTrait {
    RustTrait {
        name: "PipelineDecoderGuestContract".to_string(),
        visibility: RustVisibility::Pub,
        generic_args: Vec::new(),
        supertraits: vec!["Send".to_string(), "Sync".to_string()],
        methods: vec![signature(
            "decode",
            RustSelfKind::Ref,
            vec![param("input", "StringView")],
            Some("Result<StringView, GuestError>"),
            None,
        )],
        attributes: Vec::new(),
        docs: Some(vec![
            "Guest trait for contract `pipeline.Decoder` (id=0xE1D7DE773BE6E7F7)".to_string(),
        ]),
    }
}

fn factory_extern_block() -> RustExternBlock {
    RustExternBlock {
        abi: "Rust".to_string(),
        is_unsafe: true,
        items: vec![signature(
            "polyplug_create_decoder",
            RustSelfKind::None,
            vec![param("host", "HostContext")],
            Some("Box<dyn PipelineDecoderGuestContract>"),
            Some(vec![
                "Author-provided factory — define it in the plugin crate as:".to_string(),
                "`#[unsafe(no_mangle)]`".to_string(),
                "`pub fn polyplug_create_decoder(host: HostContext) -> Box<dyn \
                 PipelineDecoderGuestContract> { ... }`"
                    .to_string(),
            ]),
        )],
        docs: None,
    }
}

/// The `decode` ABI wrapper: FORM (this whole struct minus `body`) is langprint's; `body` is the
/// LOGIC slot polyplugc fills. The `// SAFETY` line rides in `comments` (gap 3).
fn decode_abi_form(body: Option<Vec<String>>) -> RustFunction {
    RustFunction {
        return_attributes: Vec::new(),
        name: "decoder_decode_abi".to_string(),
        visibility: RustVisibility::Private,
        self_kind: RustSelfKind::None,
        parameters: vec![
            param("instance", "GuestContractInstance"),
            param("args", "*const ()"),
            param("out", "*mut ()"),
            param("out_err", "*mut AbiError"),
        ],
        generic_args: Vec::new(),
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
        comments: vec![
            "SAFETY: args and out pointers are validated at entry before dereferencing."
                .to_string(),
        ],
    }
}

fn main() {
    let rust = RustBackend::default();

    // ---- FULL-contract FORM, byte-identical to polyplugc's golden output. ----
    let trait_src = rust
        .render_trait(&decoder_trait(), None, &mut 0)
        .expect("render trait");
    assert_eq!(
        trait_src, GOLDEN_TRAIT,
        "guest trait FORM must be byte-identical"
    );
    println!("// ===== Guest trait (byte-identical to contracts.rs) =====");
    print!("{trait_src}");
    println!();

    let extern_src = rust
        .render_extern_block(&factory_extern_block(), None, &mut 0)
        .expect("render extern block");
    assert_eq!(
        extern_src, GOLDEN_EXTERN,
        "extern factory FORM must be byte-identical"
    );
    println!("// ===== Author-factory extern block (byte-identical to interfaces.rs) =====");
    print!("{extern_src}");
    println!();

    // ---- The ABI wrapper: the FORM/LOGIC seam at `body`. ----
    let logic: Vec<String> = vec![
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
    let wrapper = rust
        .render_function(
            &decode_abi_form(Some(logic)),
            None::<&str>,
            None::<&str>,
            None,
            &mut 0,
        )
        .expect("render wrapper");
    println!("// ===== decode ABI wrapper — langprint FORM, polyplugc LOGIC (body) =====");
    print!("{wrapper}");
}
