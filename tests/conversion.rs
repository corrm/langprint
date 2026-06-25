//! Conversion tests for the scoped declaration IR.
//!
//! These exercise the `to_ir` / `from_ir` bridge: round-tripping the common declaration subset,
//! and asserting that single-language (C++-only) features are reported as `ConversionWarning`s when
//! projected to the language-agnostic IR.

use langprint::backends::BackendItem;
use langprint::backends::cpp_backend::struct_types::CppStructKind;
use langprint::backends::cpp_backend::{
    CppConstant, CppDefinition, CppEnum, CppEnumVariant, CppField, CppFunction, CppNamespace, CppParameter, CppStruct,
    CppVisibility,
};
use langprint::conversion::ConversionWarning;
use langprint::ir::{Annotation, EnumVariant, EnumVariantValue, LanguageEnum, Visibility};

fn clean_field(name: &str) -> CppField {
    CppField {
        name: name.to_string(),
        field_type: "int32_t".to_string(),
        visibility: CppVisibility::Public,
        array_size: None,
        bit_field_size: None,
        alignment: None,
        is_static: true,
        is_const: true,
        is_inline: false,
        initialization_value: None,
        inline_comment: None,
        docs: Some(vec!["a field".to_string()]),
    }
}

fn clean_function(name: &str) -> CppFunction {
    CppFunction {
        name: name.to_string(),
        parent_name: None,
        visibility: CppVisibility::Public,
        parameters: vec![CppParameter {
            name: "x".to_string(),
            param_type: "int".to_string(),
            default_value: None,
        }],
        template_params: vec![],
        return_type: Some("void".to_string()),
        is_static: false,
        is_const: false,
        is_virtual: true,
        is_pure_virtual: false,
        is_inline: false,
        is_noexcept: false,
        is_extern_c: false,
        is_override: true,
        is_final: false,
        is_friend: false,
        is_deleted: false,
        is_default: false,
        body: None,
        docs: None,
    }
}

fn scoped_enum(name: &str) -> CppEnum {
    CppEnum {
        name: name.to_string(),
        variants: vec![CppEnumVariant {
            name: "A".to_string(),
            value: Some("0".to_string()),
            docs: None,
        }],
        is_enum_class: true,
        underlying_type: Some("uint8_t".to_string()),
        docs: None,
    }
}

#[test]
fn field_round_trips_common_subset_without_warnings() {
    let ir = clean_field("count").to_ir(None);
    assert!(!ir.log.has_warnings());

    let back = CppField::from_ir(ir.value, None);
    assert!(!back.log.has_warnings());

    let field = back.value;
    assert_eq!(field.name, "count");
    assert_eq!(field.field_type, "int32_t");
    assert!(matches!(field.visibility, CppVisibility::Public));
    assert!(field.is_static);
    assert!(field.is_const);
    assert_eq!(field.docs, Some(vec!["a field".to_string()]));
    // Native-only fields lower back to their defaults.
    assert_eq!(field.array_size, None);
    assert_eq!(field.bit_field_size, None);
    assert_eq!(field.alignment, None);
    assert!(!field.is_inline);
    assert_eq!(field.initialization_value, None);
    assert_eq!(field.inline_comment, None);
}

#[test]
fn field_reports_every_dropped_cpp_feature() {
    let mut field = clean_field("flags");
    field.array_size = Some("4".to_string());
    field.bit_field_size = Some("3".to_string());
    field.alignment = Some(16);
    field.is_inline = true;
    field.initialization_value = Some("0".to_string());
    field.inline_comment = Some("0x10".to_string());

    let ir = field.to_ir(None);
    assert_eq!(
        ir.log.warnings,
        vec![
            ConversionWarning::UnsupportedFeature {
                feature: "C array dimension on field `flags`".to_string(),
                resolution: "array size dropped; encode it in the field type if needed".to_string(),
            },
            ConversionWarning::UnsupportedFeature {
                feature: "C bit-field on field `flags`".to_string(),
                resolution: "bit-field width dropped from the language-agnostic IR".to_string(),
            },
            ConversionWarning::UnsupportedFeature {
                feature: "`inline` specifier on field `flags`".to_string(),
                resolution: "inline specifier dropped from the language-agnostic IR".to_string(),
            },
            ConversionWarning::UnsupportedFeature {
                feature: "initializer on field `flags`".to_string(),
                resolution: "field initializer dropped from the language-agnostic IR".to_string(),
            },
            ConversionWarning::UnsupportedFeature {
                feature: "inline comment on field `flags`".to_string(),
                resolution: "inline comment dropped from the language-agnostic IR".to_string(),
            },
        ]
    );

    // `alignas` is now preserved as curated Tier-1 vocabulary, not dropped.
    assert_eq!(ir.value.annotations, vec![Annotation::Aligned(16)]);
}

#[test]
fn function_round_trips_signature_without_warnings() {
    let ir = clean_function("tick").to_ir(None);
    assert!(!ir.log.has_warnings());
    assert!(!ir.value.is_abstract);

    let back = CppFunction::from_ir(ir.value, None);
    assert!(!back.log.has_warnings());

    let function = back.value;
    assert_eq!(function.name, "tick");
    assert_eq!(function.return_type, Some("void".to_string()));
    assert!(function.is_virtual);
    assert!(function.is_override);
    assert_eq!(function.parameters.len(), 1);
    assert_eq!(function.parameters[0].name, "x");
    // `from_ir` canonicalizes primitive spellings to the target language via the TypeMap.
    assert_eq!(function.parameters[0].param_type, "int32_t");
    // Native-only modifiers lower back to their defaults.
    assert!(!function.is_const);
    assert!(!function.is_noexcept);
    assert!(!function.is_friend);
}

#[test]
fn function_reports_every_dropped_cpp_modifier() {
    let mut function = clean_function("op");
    function.is_const = true;
    function.is_inline = true;
    function.is_noexcept = true;
    function.is_friend = true;
    function.is_deleted = true;
    function.is_default = true;

    let ir = function.to_ir(None);
    let dropped = "modifier dropped from the language-agnostic IR".to_string();
    assert_eq!(
        ir.log.warnings,
        vec![
            ConversionWarning::UnsupportedFeature {
                feature: "C++ `const` member function on `op`".to_string(),
                resolution: dropped.clone(),
            },
            ConversionWarning::UnsupportedFeature {
                feature: "C++ `inline` specifier on `op`".to_string(),
                resolution: dropped.clone(),
            },
            ConversionWarning::UnsupportedFeature {
                feature: "C++ `noexcept` specifier on `op`".to_string(),
                resolution: dropped.clone(),
            },
            ConversionWarning::UnsupportedFeature {
                feature: "C++ `friend` function on `op`".to_string(),
                resolution: dropped.clone(),
            },
            ConversionWarning::UnsupportedFeature {
                feature: "C++ `= delete` on `op`".to_string(),
                resolution: dropped.clone(),
            },
            ConversionWarning::UnsupportedFeature {
                feature: "C++ `= default` on `op`".to_string(),
                resolution: dropped,
            },
        ]
    );
}

#[test]
fn pure_virtual_maps_to_abstract_both_ways() {
    let mut function = clean_function("draw");
    function.is_pure_virtual = true;

    let ir = function.to_ir(None);
    // pure-virtual is representable as abstract, so it is not a dropped feature.
    assert!(!ir.log.has_warnings());
    assert!(ir.value.is_abstract);

    let back = CppFunction::from_ir(ir.value, None).value;
    assert!(back.is_pure_virtual);
    assert!(back.is_virtual);
}

#[test]
fn unscoped_enum_is_reported() {
    let mut cpp_enum = scoped_enum("Color");
    cpp_enum.is_enum_class = false;

    let ir = cpp_enum.to_ir(None);
    assert_eq!(ir.log.warnings.len(), 1);
    assert!(ir.log.warnings.iter().any(|w| matches!(
        w,
        ConversionWarning::UnsupportedFeature { feature, .. } if feature == "unscoped C++ enum `Color`"
    )));
}

#[test]
fn scoped_enum_round_trips_without_warnings() {
    let ir = scoped_enum("Color").to_ir(None);
    assert!(!ir.log.has_warnings());

    let back = CppEnum::from_ir(ir.value, None);
    assert!(!back.log.has_warnings());
    // from_ir lowers to an idiomatic scoped enum.
    assert!(back.value.is_enum_class);
    assert_eq!(back.value.name, "Color");
    assert_eq!(back.value.underlying_type, Some("uint8_t".to_string()));
}

#[test]
fn from_ir_defaults_enum_to_scoped() {
    let language_enum = LanguageEnum {
        name: "Flags".to_string(),
        visibility: Visibility::Default,
        variants: vec![EnumVariant {
            name: "None".to_string(),
            value: EnumVariantValue::Value("0".to_string()),
            docs: None,
        }],
        underlying_type: None,
        docs: None,
        annotations: Vec::new(),
        raw_attributes: Vec::new(),
    };

    let back = CppEnum::from_ir(language_enum, None);
    assert!(back.value.is_enum_class);
}

#[test]
fn parameter_round_trips() {
    let param = CppParameter {
        name: "id".to_string(),
        param_type: "uint32_t".to_string(),
        default_value: Some("0".to_string()),
    };

    let ir = param.to_ir(None);
    assert!(!ir.log.has_warnings());

    let back = CppParameter::from_ir(ir.value, None).value;
    assert_eq!(back.name, "id");
    assert_eq!(back.param_type, "uint32_t");
    assert_eq!(back.default_value, Some("0".to_string()));
}

#[test]
fn namespace_round_trips_nested_members_without_warnings() {
    let inner = CppNamespace {
        name: "inner".to_string(),
        defines: Some(vec![CppDefinition {
            name: "VERSION".to_string(),
            value: Some("1".to_string()),
            docs: None,
        }]),
        constants: None,
        enums: Some(vec![scoped_enum("Mode")]),
        structs: None,
        functions: None,
        namespaces: None,
    };

    let outer = CppNamespace {
        name: "sdk".to_string(),
        defines: None,
        constants: Some(vec![CppConstant {
            name: "MAX".to_string(),
            visibility: CppVisibility::Public,
            data_type: "int".to_string(),
            value: "10".to_string(),
            docs: None,
        }]),
        enums: None,
        structs: Some(vec![CppStruct {
            struct_kind: CppStructKind::Struct,
            is_final: false,
            alignment: None,
            name: "Vec2".to_string(),
            template_params: vec![],
            bases: vec![],
            fields: vec![clean_field("x")],
            methods: vec![clean_function("len")],
            docs: None,
        }]),
        functions: None,
        namespaces: Some(vec![inner]),
    };

    let ir = outer.to_ir(None);
    assert!(!ir.log.has_warnings());

    let back = CppNamespace::from_ir(ir.value, None);
    assert!(!back.log.has_warnings());

    let namespace = back.value;
    assert_eq!(namespace.name, "sdk");
    assert_eq!(namespace.constants.as_ref().map(Vec::len), Some(1));
    assert_eq!(namespace.structs.as_ref().map(Vec::len), Some(1));
    let nested = namespace.namespaces.expect("nested namespace preserved");
    assert_eq!(nested.len(), 1);
    assert_eq!(nested[0].name, "inner");
    assert_eq!(nested[0].defines.as_ref().map(Vec::len), Some(1));
    assert_eq!(nested[0].enums.as_ref().map(Vec::len), Some(1));
}

fn struct_with_methods(name: &str, methods: Vec<CppFunction>) -> CppStruct {
    CppStruct {
        struct_kind: CppStructKind::Class,
        is_final: false,
        alignment: None,
        name: name.to_string(),
        template_params: vec![],
        bases: vec![],
        fields: vec![],
        methods,
        docs: None,
    }
}

#[test]
fn struct_is_abstract_is_derived_from_a_pure_virtual_method() {
    // A C++ type is abstract iff it declares at least one pure-virtual method.
    let mut pure = clean_function("draw");
    pure.is_pure_virtual = true;
    let abstract_struct = struct_with_methods("Shape", vec![pure]).to_ir(None).value;
    assert!(abstract_struct.is_abstract);

    let concrete_struct = struct_with_methods("Square", vec![clean_function("draw")])
        .to_ir(None)
        .value;
    assert!(!concrete_struct.is_abstract);
}
