//! Tests for the two-tier IR annotation system (LANGPRINT-10).
//!
//! The IR no longer blanket-drops native attributes/derives/repr. Tier 1 (`Annotation`) is a
//! curated source-neutral vocabulary; Tier 2 (`RawAttribute`) carries opaque source-tagged
//! attributes that round-trip losslessly within their own language and are warned-and-dropped
//! when projected to a different target.

use langprint::backends::BackendItem;
use langprint::backends::csharp_backend::CSharpType;
use langprint::backends::rust_backend::{RustStruct, RustVisibility};
use langprint::conversion::ConversionWarning;
use langprint::ir::{Annotation, RawAttribute};
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
