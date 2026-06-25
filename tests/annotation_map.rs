//! Tests for the configurable forward annotation lowering ([`AnnotationMap`]).
//!
//! The builtin map reproduces the previously-hardcoded Rust/C# tables byte-for-byte; callers can
//! clone it and override or extend a `(language, kind)` spelling before driving a `from_ir`.

use langprint::backends::BackendItem;
use langprint::backends::rust_backend::{RustStruct, RustStructConversionOptions, RustVisibility};
use langprint::ir::{Annotation, AnnotationKind};
use langprint::type_map::TargetLanguage;
use langprint::{AnnotationMap, ConversionConfig};

#[test]
fn annotation_map_builtin_rust() {
    let map = AnnotationMap::default();
    assert_eq!(map.resolve(TargetLanguage::Rust, &Annotation::ReprC).as_deref(), Some("repr(C)"));
    assert_eq!(map.resolve(TargetLanguage::Rust, &Annotation::Packed).as_deref(), Some("repr(packed)"));
    assert_eq!(map.resolve(TargetLanguage::Rust, &Annotation::Aligned(8)).as_deref(), Some("repr(align(8))"));
}

#[test]
fn annotation_map_builtin_csharp() {
    let map = AnnotationMap::default();
    assert_eq!(
        map.resolve(TargetLanguage::CSharp, &Annotation::ReprC).as_deref(),
        Some("StructLayout(LayoutKind.Sequential)")
    );
    assert_eq!(
        map.resolve(TargetLanguage::CSharp, &Annotation::Packed).as_deref(),
        Some("StructLayout(LayoutKind.Sequential, Pack = 1)")
    );
    assert_eq!(map.resolve(TargetLanguage::CSharp, &Annotation::Aligned(8)), None);
}

#[test]
fn annotation_map_empty_resolves_none() {
    let map = AnnotationMap::empty();
    assert_eq!(map.resolve(TargetLanguage::Rust, &Annotation::ReprC), None);
}

#[test]
fn annotation_map_override_in_from_ir() {
    let mut annotation_map = AnnotationMap::default();
    annotation_map.insert(TargetLanguage::Rust, AnnotationKind::ReprC, "repr(C, packed)");

    let config = ConversionConfig {
        annotation_map,
        ..ConversionConfig::default()
    };
    let options = RustStructConversionOptions { config };

    let input = RustStruct {
        name: "Packet".to_string(),
        visibility: RustVisibility::Pub,
        generic_args: vec![],
        fields: vec![],
        methods: vec![],
        derives: vec![],
        attributes: vec!["repr(C)".to_string()],
        is_tuple: false,
        docs: None,
    };

    let ir = input.to_ir(None);
    let back = RustStruct::from_ir(ir.value, Some(&options));

    assert!(back.value.attributes.contains(&"repr(C, packed)".to_string()));
}
