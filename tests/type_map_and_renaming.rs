//! Exact-value tests for the cross-language TypeMap (B1), idiomatic renaming (S8), and the
//! generic-loss reporting (S2).

use langprint::backends::BackendItem;
use langprint::backends::csharp_backend::{CSharpField, CSharpFieldConversionOptions, CSharpType};
use langprint::backends::rust_backend::RustField;
use langprint::conversion::ConversionWarning;
use langprint::ir::{LanguageField, LanguageGenericArgument, LanguageStruct, LanguageStructKind, Visibility};
use langprint::naming::{to_pascal_case, to_snake_case};
use langprint::{ConversionConfig, PrimitiveType, TargetLanguage, TypeMap};

fn ir_field(name: &str, field_type: &str) -> LanguageField {
    LanguageField {
        name: name.to_string(),
        field_type: field_type.to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_const: false,
        docs: None,
        annotations: Vec::new(),
        raw_attributes: Vec::new(),
    }
}

#[test]
fn builtin_maps_primitives_across_languages() {
    let map = TypeMap::builtin();
    assert_eq!(map.map("f32", TargetLanguage::CSharp), Some("float".to_string()));
    assert_eq!(map.map("uint8_t", TargetLanguage::Rust), Some("u8".to_string()));
    assert_eq!(map.map("byte", TargetLanguage::Rust), Some("u8".to_string()));
    assert_eq!(map.map("int", TargetLanguage::Cpp), Some("int32_t".to_string()));
    assert_eq!(map.map("double", TargetLanguage::Rust), Some("f64".to_string()));
    assert_eq!(map.resolve("ushort"), Some(PrimitiveType::U16));
    // A user-defined type is not a primitive.
    assert_eq!(map.map("Player", TargetLanguage::Rust), None);
}

#[test]
fn type_map_override_extend_and_clear() {
    let mut map = TypeMap::builtin();
    map.set_output(PrimitiveType::F32, TargetLanguage::CSharp, "Single");
    assert_eq!(map.map("f32", TargetLanguage::CSharp), Some("Single".to_string()));

    map.insert_spelling("FFloat", PrimitiveType::F32);
    assert_eq!(map.resolve("FFloat"), Some(PrimitiveType::F32));

    let mut extension = TypeMap::empty();
    extension.insert_spelling("BOOL", PrimitiveType::Bool);
    extension.set_output(PrimitiveType::Bool, TargetLanguage::Rust, "bool");
    map.extend(extension);
    assert_eq!(map.map("BOOL", TargetLanguage::Rust), Some("bool".to_string()));

    map.clear();
    assert_eq!(map.map("f32", TargetLanguage::CSharp), None);
    assert_eq!(map.resolve("int"), None);
}

#[test]
fn unmapped_field_type_is_verbatim_with_warning() {
    let result = RustField::from_ir(ir_field("owner", "Player"), None);
    assert_eq!(result.value.field_type, "Player");
    assert_eq!(
        result.log.warnings,
        vec![ConversionWarning::UnsupportedFeature {
            feature: "unmapped type `Player`".to_string(),
            resolution: "no TypeMap entry for Rust; emitted verbatim".to_string(),
        }]
    );
}

#[test]
fn csharp_field_is_pascal_cased_with_warning() {
    let result = CSharpField::from_ir(ir_field("health_points", "f32"), None);
    assert_eq!(result.value.name, "HealthPoints");
    assert_eq!(result.value.field_type, "float");
    assert_eq!(
        result.log.warnings,
        vec![ConversionWarning::NamingConventionChanged {
            original: "health_points".to_string(),
            converted: "HealthPoints".to_string(),
        }]
    );
}

#[test]
fn rename_disabled_keeps_identifier_but_still_maps_type() {
    let options = CSharpFieldConversionOptions {
        config: ConversionConfig::new(TypeMap::builtin(), false),
    };
    let result = CSharpField::from_ir(ir_field("health_points", "f32"), Some(&options));
    // Identifier is left verbatim...
    assert_eq!(result.value.name, "health_points");
    // ...but type mapping still applies.
    assert_eq!(result.value.field_type, "float");
    assert!(!result.log.has_warnings());
}

#[test]
fn csharp_generic_default_is_reported() {
    let ir = LanguageStruct {
        visibility: Visibility::Public,
        struct_kind: LanguageStructKind::Class,
        is_abstract: false,
        is_final: false,
        name: "Container".to_string(),
        generic_args: vec![LanguageGenericArgument {
            name: "T".to_string(),
            keyword: String::new(),
            default_value: Some("int".to_string()),
            where_clause: None,
        }],
        bases: vec![],
        fields: vec![],
        methods: vec![],
        docs: None,
        annotations: Vec::new(),
        raw_attributes: Vec::new(),
    };

    let result = CSharpType::from_ir(ir, None);
    assert_eq!(
        result.log.warnings,
        vec![ConversionWarning::UnsupportedFeature {
            feature: "default on generic parameter `T`".to_string(),
            resolution: "C# has no generic parameter defaults; the default was dropped".to_string(),
        }]
    );
}

#[test]
fn naming_case_conversions() {
    assert_eq!(to_snake_case("HealthPoints"), "health_points");
    assert_eq!(to_snake_case("XMLHttpRequest"), "xml_http_request");
    assert_eq!(to_pascal_case("health_points"), "HealthPoints");
    assert_eq!(to_pascal_case("get-view-point"), "GetViewPoint");
}
