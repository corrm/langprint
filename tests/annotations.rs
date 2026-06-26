//! Tests for the two-tier IR annotation system (LANGPRINT-10).
//!
//! The IR no longer blanket-drops native attributes/derives/repr. Tier 1 (`Annotation`) is a
//! curated source-neutral vocabulary; Tier 2 (`RawAttribute`) carries opaque source-tagged
//! attributes that round-trip losslessly within their own language and are warned-and-dropped
//! when projected to a different target.

use langprint::backends::BackendItem;
use langprint::backends::cpp_backend::{CppBackend, CppStruct, DocsStyle};
use langprint::backends::csharp_backend::{CSharpBackend, CSharpType, CSharpTypeKind, CSharpVisibility};
use langprint::backends::rust_backend::{RustBackend, RustStruct, RustVisibility};
use langprint::conversion::ConversionWarning;
use langprint::ir::{Annotation, LanguageField, LanguageStruct, LanguageStructKind, RawAttribute, Visibility};
use langprint::renderers::StructRenderer;
use langprint::text::{IndentStyle, NewLineStyle};
use langprint::type_map::TargetLanguage;

fn rust_struct(derives: Vec<&str>, attributes: Vec<&str>) -> RustStruct {
    RustStruct {
        name: "Packet".to_string(),
        visibility: RustVisibility::Pub,
        generic_args: vec![],
        fields: vec![],
        methods: vec![],
        derives: derives.into_iter().map(str::to_string).collect(),
        attributes: attributes.into_iter().map(str::to_string).collect(),
        is_tuple: false,
        docs: None,
    }
}

fn ir_struct(annotations: Vec<Annotation>) -> LanguageStruct {
    LanguageStruct {
        visibility: Visibility::Public,
        struct_kind: LanguageStructKind::Struct,
        is_abstract: false,
        is_final: false,
        name: "Packet".to_string(),
        generic_args: Vec::new(),
        bases: Vec::new(),
        fields: vec![LanguageField {
            name: "x".to_string(),
            field_type: "i32".to_string(),
            visibility: Visibility::Public,
            is_static: false,
            is_const: false,
            docs: None,
            annotations: Vec::new(),
            raw_attributes: Vec::new(),
        }],
        methods: Vec::new(),
        docs: None,
        annotations,
        raw_attributes: Vec::new(),
    }
}

fn render_rust(value: &RustStruct) -> String {
    RustBackend::default()
        .render_struct(value, None::<&str>, None::<&str>, None, &mut 0)
        .unwrap()
}

fn cpp_backend() -> CppBackend {
    CppBackend {
        new_line: NewLineStyle::LF,
        open_brace_on_new_line: false,
        docs_style: DocsStyle::DoubleSlash,
        indent_style: IndentStyle::Spaces,
        indent_size: 4,
    }
}

/// Tier-1 `Packed` lowers to the idiomatic packed-layout attribute in Rust and C#.
#[test]
fn tier1_packed_lowers_to_rust_and_csharp() {
    let rust = RustStruct::from_ir(ir_struct(vec![Annotation::Packed]), None);
    assert!(
        render_rust(&rust.value).contains("#[repr(packed)]"),
        "rendered: {}",
        render_rust(&rust.value)
    );

    let csharp = CSharpType::from_ir(ir_struct(vec![Annotation::Packed]), None);
    let rendered = CSharpBackend::default()
        .render_struct::<&str>(&csharp.value, None, None, None, &mut 0)
        .unwrap();
    assert!(
        rendered.contains("StructLayout(LayoutKind.Sequential, Pack = 1)"),
        "rendered: {rendered}"
    );
}

/// Tier-1 `Aligned(8)` lowers to `repr(align(8))` in Rust and to numeric `alignas(8)` in C++.
#[test]
fn tier1_aligned_lowers_to_rust_and_cpp() {
    let rust = RustStruct::from_ir(ir_struct(vec![Annotation::Aligned(8)]), None);
    assert!(
        render_rust(&rust.value).contains("#[repr(align(8))]"),
        "rendered: {}",
        render_rust(&rust.value)
    );

    let cpp = CppStruct::from_ir(ir_struct(vec![Annotation::Aligned(8)]), None);
    assert_eq!(cpp.value.alignment, Some(8));
    let rendered = cpp_backend()
        .render_struct::<&str>(&cpp.value, None, None, None, &mut 0)
        .unwrap();
    assert!(rendered.contains("alignas(8)"), "rendered: {rendered}");
}

/// Tier-2 cross-language in a non-`Rust`→`C#` direction: a C#-only opaque attribute is dropped and
/// warned when lowered to Rust.
#[test]
fn tier2_csharp_only_attribute_is_dropped_for_rust_with_warning() {
    let source = CSharpType {
        kind: CSharpTypeKind::Struct,
        name: "Packet".to_string(),
        visibility: CSharpVisibility::Public,
        is_abstract: false,
        is_sealed: false,
        is_static: false,
        is_unsafe: false,
        is_partial: false,
        generic_args: vec![],
        base_class: None,
        interfaces: vec![],
        fields: vec![],
        properties: vec![],
        methods: vec![],
        attributes: vec!["Serializable".to_string()],
        docs: None,
    };

    let ir = source.to_ir(None);
    assert_eq!(
        ir.value.raw_attributes,
        vec![RawAttribute {
            source: TargetLanguage::CSharp,
            text: "Serializable".to_string(),
        }]
    );

    let rust = RustStruct::from_ir(ir.value, None);
    assert!(
        rust.value.attributes.is_empty() && rust.value.derives.is_empty(),
        "C#-only attribute must not appear in Rust, got attrs {:?} derives {:?}",
        rust.value.attributes,
        rust.value.derives
    );
    assert!(
        rust.log.warnings.iter().any(|warning| matches!(
            warning,
            ConversionWarning::UnsupportedFeature { feature, .. } if feature.contains("CSharp") && feature.contains("Serializable")
        )),
        "expected a dropped-attribute warning naming the C# source, got {:?}",
        rust.log.warnings
    );
}

/// Same-language round-trip preserves a previously-dropped attribute. This is the headline fix:
/// the blanket drop is gone, so a Rust derive survives `to_ir` → `from_ir` back into Rust.
#[test]
fn same_language_roundtrip_preserves_dropped_derive() {
    let ir = rust_struct(vec!["Clone"], vec![]).to_ir(None);
    assert!(ir.log.warnings.is_empty());

    let back = RustStruct::from_ir(ir.value, None);
    assert!(back.log.warnings.is_empty());
    assert_eq!(back.value.derives, vec!["Clone".to_string()]);
}

/// Tier-1 cross-language: a Rust `repr(C)` becomes the source-neutral `Annotation::ReprC`, which
/// the C# backend re-emits as `[StructLayout(LayoutKind.Sequential)]`.
#[test]
fn tier1_repr_c_crosses_rust_to_csharp() {
    let ir = rust_struct(vec![], vec!["repr(C)"]).to_ir(None);
    assert_eq!(ir.value.annotations, vec![Annotation::ReprC]);

    let csharp = CSharpType::from_ir(ir.value, None);
    assert!(
        csharp
            .value
            .attributes
            .iter()
            .any(|attribute| attribute == "StructLayout(LayoutKind.Sequential)"),
        "expected StructLayout attribute, got {:?}",
        csharp.value.attributes
    );
}

/// Tier-2 cross-language: a Rust-only opaque attribute cannot translate to C#, so it is dropped and
/// a warning is logged identifying the Rust source.
#[test]
fn tier2_rust_only_attribute_is_dropped_for_csharp_with_warning() {
    let ir = rust_struct(vec!["Clone"], vec![]).to_ir(None);
    assert_eq!(
        ir.value.raw_attributes,
        vec![RawAttribute {
            source: TargetLanguage::Rust,
            text: "derive(Clone)".to_string(),
        }]
    );

    let csharp = CSharpType::from_ir(ir.value, None);
    assert!(
        csharp.value.attributes.is_empty(),
        "Rust-only attribute should not appear in C#, got {:?}",
        csharp.value.attributes
    );
    assert!(
        csharp.log.warnings.iter().any(|warning| matches!(
            warning,
            ConversionWarning::UnsupportedFeature { feature, .. } if feature.contains("Rust") && feature.contains("derive(Clone)")
        )),
        "expected a dropped-attribute warning naming the Rust source, got {:?}",
        csharp.log.warnings
    );
}
