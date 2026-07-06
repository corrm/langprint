//! Rust `trait` + `extern` block FORM, and bare `//` function comments.
//!
//! The two golden constants are the exact bytes polyplugc's Rust generator emits for the
//! `pipeline.Decoder` guest contract
//! (`polyplug/examples/guests/rust/decoder/generated/guest/{contracts,interfaces}.rs`). langprint
//! must reproduce them byte-for-byte — this is the "Rust FORM is complete" acceptance for the
//! polyplugc seam.

use langprint::backends::BackendItem;
use langprint::backends::rust_backend::{
    RustBackend, RustExternBlock, RustFunction, RustParameter, RustSelfKind, RustTrait,
    RustVisibility,
};
use langprint::conversion::ConversionWarning;
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
        name: name.to_string(),
        param_type: ty.to_string(),
    }
}

fn signature(
    name: &str,
    self_kind: RustSelfKind,
    parameters: Vec<RustParameter>,
    return_type: Option<&str>,
    docs: Option<Vec<String>>,
) -> RustFunction {
    RustFunction {
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

#[test]
fn guest_trait_form_is_byte_identical() {
    let decoder_trait = RustTrait {
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
    };
    let rendered = RustBackend::default()
        .render_trait(&decoder_trait, None, &mut 0)
        .expect("render trait");
    assert_eq!(rendered, GOLDEN_TRAIT);
}

#[test]
fn author_factory_extern_block_is_byte_identical() {
    let block = RustExternBlock {
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
    };
    let rendered = RustBackend::default()
        .render_extern_block(&block, None, &mut 0)
        .expect("render extern block");
    assert_eq!(rendered, GOLDEN_EXTERN);
}

#[test]
fn empty_trait_renders_unit_block() {
    let empty = RustTrait {
        name: "Marker".to_string(),
        visibility: RustVisibility::Pub,
        generic_args: Vec::new(),
        supertraits: Vec::new(),
        methods: Vec::new(),
        attributes: Vec::new(),
        docs: None,
    };
    let rendered = RustBackend::default()
        .render_trait(&empty, None, &mut 0)
        .expect("render trait");
    assert_eq!(rendered, "pub trait Marker {}\n");
}

#[test]
fn bare_line_comment_renders_between_docs_and_attributes() {
    let func = RustFunction {
        comments: vec!["SAFETY: caller upholds the invariant.".to_string()],
        attributes: vec!["inline".to_string()],
        docs: Some(vec!["A documented function.".to_string()]),
        ..signature("f", RustSelfKind::None, Vec::new(), None, None)
    };
    let rendered = RustBackend::default()
        .render_function(&func, None::<&str>, None::<&str>, None, &mut 0)
        .expect("render function");
    assert_eq!(
        rendered,
        "/// A documented function.\n// SAFETY: caller upholds the invariant.\n#[inline]\nfn f();\n"
    );
}

#[test]
fn bare_line_comments_warn_when_lowered_to_ir() {
    let func = RustFunction {
        comments: vec!["a line comment".to_string()],
        ..signature("f", RustSelfKind::None, Vec::new(), None, None)
    };
    let warnings = func.to_ir(None).log.warnings;
    assert!(
        warnings.iter().any(|w| matches!(
            w,
            ConversionWarning::UnsupportedFeature { feature, .. } if feature.contains("comment")
        )),
        "expected a dropped-comment warning, got: {warnings:?}"
    );
}
