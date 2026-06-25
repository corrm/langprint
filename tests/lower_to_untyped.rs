//! Lowering tests: a neutral IR sourced from a typed backend lowers INTO idiomatic Python/Lua/JS
//! via each untyped backend's `from_ir`. This is the "from_ir render target" direction — types are
//! re-spelled (Python PEP-484 hints, JS JSDoc, Lua none) and identifiers renamed to convention.
//! The reverse (untyped → IR transpile) is explicitly out of scope and not tested here.

use langprint::backends::js_backend::{JsClass, JsFunction};
use langprint::backends::lua_backend::{LuaBackend, LuaFunction};
use langprint::backends::python_backend::{
    ctypes_type_map, PythonBackend, PythonClass, PythonFunction, PythonStruct, PythonStructConversionOptions,
};
use langprint::backends::BackendItem;
use langprint::conversion::ConversionWarning;
use langprint::convert::ConversionConfig;
use langprint::ir::{
    Annotation, LanguageBase, LanguageField, LanguageFunction, LanguageFunctionParameter, LanguageStruct,
    LanguageStructKind, RawAttribute, Visibility,
};
use langprint::renderers::{FunctionRenderer, StructRenderer};
use langprint::type_map::{PrimitiveType, TargetLanguage, TypeMap};

/// Build a ConversionConfig with ctypes spellings for Python output.
fn ctypes_config() -> ConversionConfig {
    ConversionConfig::new(ctypes_type_map(), false)
}

/// Build a ConversionConfig with ctypes spellings plus custom overrides.
fn ctypes_config_with_override(overrides: &[(PrimitiveType, &str)], custom_types: Vec<(&str, &str)>) -> ConversionConfig {
    let mut type_map = ctypes_type_map();
    for (primitive, spelling) in overrides {
        type_map.set_output(*primitive, TargetLanguage::Python, *spelling);
    }
    let mut config = ConversionConfig::new(type_map, false);
    let custom: Vec<(String, String)> = custom_types.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    config.type_override = Some(std::sync::Arc::new(move |spelling: &str, lang: TargetLanguage| -> Option<String> {
        if lang != TargetLanguage::Python {
            return None;
        }
        custom.iter().find(|(s, _)| s.as_str() == spelling).map(|(_, v)| v.clone())
    }));
    config
}

/// A neutral function as if projected from a typed backend: PascalCase name, `i32`/`f64` params,
/// an `f64` return. Lowering must rename to the target convention and re-spell the types.
fn neutral_function() -> LanguageFunction {
    LanguageFunction {
        name: "ComputeTotal".to_string(),
        visibility: Visibility::Public,
        parameters: vec![
            LanguageFunctionParameter {
                name: "ItemCount".to_string(),
                param_type: "i32".to_string(),
                default_value: None,
            },
            LanguageFunctionParameter {
                name: "UnitPrice".to_string(),
                param_type: "f64".to_string(),
                default_value: None,
            },
        ],
        generic_args: Vec::new(),
        return_type: Some("f64".to_string()),
        is_static: true,
        is_abstract: false,
        is_virtual: false,
        is_override: false,
        is_final: false,
        body: None,
        docs: None,
        annotations: Vec::new(),
        raw_attributes: Vec::new(),
    }
}

/// A neutral class as if projected from a typed backend: PascalCase name, a snake-able field, one
/// method, and two bases (to exercise JS single-inheritance collapse).
fn neutral_struct() -> LanguageStruct {
    LanguageStruct {
        visibility: Visibility::Public,
        struct_kind: LanguageStructKind::Class,
        is_abstract: false,
        is_final: false,
        name: "OrderLine".to_string(),
        generic_args: Vec::new(),
        bases: vec![
            LanguageBase {
                name: "BaseEntity".to_string(),
                visibility: Visibility::Public,
            },
            LanguageBase {
                name: "Auditable".to_string(),
                visibility: Visibility::Public,
            },
        ],
        fields: vec![LanguageField {
            name: "TotalAmount".to_string(),
            field_type: "f64".to_string(),
            visibility: Visibility::Public,
            is_static: false,
            is_const: false,
            docs: None,
            annotations: Vec::new(),
            raw_attributes: Vec::new(),
        }],
        methods: vec![neutral_function()],
        docs: None,
        annotations: Vec::new(),
        raw_attributes: Vec::new(),
    }
}

#[test]
fn lowers_function_to_python_snake_case_with_pep484_hints() {
    let function = PythonFunction::from_ir(neutral_function(), None).value;

    assert_eq!(function.name, "compute_total");
    assert_eq!(function.parameters[0].name, "item_count");
    assert_eq!(function.parameters[0].type_hint.as_deref(), Some("int"));
    assert_eq!(function.parameters[1].name, "unit_price");
    assert_eq!(function.parameters[1].type_hint.as_deref(), Some("float"));
    assert_eq!(function.return_type.as_deref(), Some("float"));
}

#[test]
fn lowers_class_to_python_pascal_with_snake_fields() {
    let class = PythonClass::from_ir(neutral_struct(), None).value;

    assert_eq!(class.name, "OrderLine");
    assert_eq!(class.fields[0].name, "total_amount");
    assert_eq!(class.methods[0].name, "compute_total");
    assert_eq!(class.bases, vec!["BaseEntity".to_string(), "Auditable".to_string()]);
}

#[test]
fn lowers_function_to_js_camel_case_with_jsdoc_number() {
    let function = JsFunction::from_ir(neutral_function(), None).value;

    assert_eq!(function.name, "computeTotal");
    assert_eq!(function.parameters[0].name, "itemCount");
    // Signature stays untyped — types live only in `type_doc` for JSDoc.
    assert_eq!(function.parameters[0].type_doc.as_deref(), Some("number"));
    assert_eq!(function.parameters[1].name, "unitPrice");
    assert_eq!(function.parameters[1].type_doc.as_deref(), Some("number"));
    assert_eq!(function.return_type.as_deref(), Some("number"));
}

#[test]
fn lowers_class_to_js_single_extends_with_warning() {
    let result = JsClass::from_ir(neutral_struct(), None);
    let class = &result.value;

    assert_eq!(class.name, "OrderLine");
    assert_eq!(class.fields[0].name, "totalAmount");
    assert_eq!(class.methods[0].name, "computeTotal");
    // Two bases collapse to the first; the drop is reported, never silent.
    assert_eq!(class.extends.as_deref(), Some("BaseEntity"));
    assert!(result
        .log
        .warnings
        .iter()
        .any(|warning| matches!(warning, langprint::conversion::ConversionWarning::UnsupportedFeature { .. })));
}

/// A neutral struct (not a class) with primitive fields, as if destined for a ctypes `Structure`.
fn neutral_ctypes_struct(fields: Vec<LanguageField>) -> LanguageStruct {
    LanguageStruct {
        visibility: Visibility::Public,
        struct_kind: LanguageStructKind::Struct,
        is_abstract: false,
        is_final: false,
        name: "Vec2".to_string(),
        generic_args: Vec::new(),
        bases: Vec::new(),
        fields,
        methods: Vec::new(),
        docs: None,
        annotations: Vec::new(),
        raw_attributes: Vec::new(),
    }
}

fn primitive_field(name: &str, field_type: &str) -> LanguageField {
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

fn render_python_struct(value: &PythonStruct) -> String {
    let backend = PythonBackend::default();
    let mut level = 0;
    backend
        .render_struct(value, None::<&str>, None::<&str>, None, &mut level)
        .unwrap()
}

#[test]
fn python_struct_from_ir_maps_primitives_to_ctypes() {
    let input = neutral_ctypes_struct(vec![
        primitive_field("X", "f64"),
        primitive_field("Count", "i32"),
    ]);
    let options = PythonStructConversionOptions { config: ctypes_config() };
    let result = PythonStruct::from_ir(input, Some(&options));
    let rendered = render_python_struct(&result.value);

    assert!(rendered.contains("ctypes.c_double"), "rendered: {rendered}");
    assert!(rendered.contains("ctypes.c_int32"), "rendered: {rendered}");
    assert!(!rendered.contains("f64"), "literal IR spelling leaked: {rendered}");
    assert!(!rendered.contains("i32"), "literal IR spelling leaked: {rendered}");
}

#[test]
fn ctypes_type_map_has_expected_entries() {
    let map = ctypes_type_map();
    assert_eq!(map.map("f64", TargetLanguage::Python), Some("ctypes.c_double".to_string()));
    assert_eq!(map.map("i32", TargetLanguage::Python), Some("ctypes.c_int32".to_string()));
    // i128 has no ctypes entry — falls back to builtin TypeMap Python output
    assert_eq!(map.map("i128", TargetLanguage::Python), Some("int".to_string()));
}

#[test]
fn python_struct_from_ir_uses_custom_type_map() {
    let config = ctypes_config_with_override(
        &[(PrimitiveType::F64, "MyDouble"), (PrimitiveType::I128, "ctypes.c_int128")],
        vec![],
    );
    let options = PythonStructConversionOptions { config };

    let input = neutral_ctypes_struct(vec![
        primitive_field("X", "f64"),
        primitive_field("Big", "i128"),
    ]);
    let result = PythonStruct::from_ir(input, Some(&options));
    let rendered = render_python_struct(&result.value);

    assert!(rendered.contains("MyDouble"), "rendered: {rendered}");
    assert!(rendered.contains("ctypes.c_int128"), "rendered: {rendered}");
    assert!(
        !result
            .log
            .warnings
            .iter()
            .any(|warning| matches!(warning, ConversionWarning::UnsupportedFeature { .. })),
        "supplied i128 mapping must not warn: {:?}",
        result.log.warnings
    );
}

#[test]
fn python_struct_from_ir_default_options_uses_builtin_typemap() {
    let input = neutral_ctypes_struct(vec![
        primitive_field("X", "f64"),
        primitive_field("Count", "i32"),
    ]);
    let result = PythonStruct::from_ir(input, None);
    let rendered = render_python_struct(&result.value);

    // Default TypeMap maps f64→float, i32→int for Python (PEP-484 hints).
    assert!(rendered.contains("float"), "rendered: {rendered}");
    assert!(rendered.contains("int"), "rendered: {rendered}");
}

#[test]
fn python_struct_from_ir_custom_type_override() {
    let config = ctypes_config_with_override(
        &[],
        vec![("MyHandle", "ctypes.c_void_p")],
    );
    let options = PythonStructConversionOptions { config };

    let input = neutral_ctypes_struct(vec![primitive_field("Handle", "MyHandle")]);
    let result = PythonStruct::from_ir(input, Some(&options));
    let rendered = render_python_struct(&result.value);

    assert!(rendered.contains("ctypes.c_void_p"), "rendered: {rendered}");
    assert!(
        !rendered.contains("MyHandle"),
        "custom type must be mapped, not passed verbatim: {rendered}"
    );
    assert!(
        !result
            .log
            .warnings
            .iter()
            .any(|warning| matches!(warning, ConversionWarning::UnsupportedFeature { .. })),
        "a mapped custom type must not warn as unsupported: {:?}",
        result.log.warnings
    );
}

#[test]
fn python_struct_from_ir_passes_unknown_ctype_verbatim() {
    // Unknown types pass through verbatim with a warning. The user provides a
    // type_override to map custom types and suppress the warning.
    let config = ctypes_config_with_override(
        &[],
        vec![("ctypes.c_void_p", "ctypes.c_void_p"), ("SomeStructure", "SomeStructure")],
    );
    let options = PythonStructConversionOptions { config };

    let input = neutral_ctypes_struct(vec![
        primitive_field("Handle", "ctypes.c_void_p"),
        primitive_field("Nested", "SomeStructure"),
    ]);
    let result = PythonStruct::from_ir(input, Some(&options));
    let rendered = render_python_struct(&result.value);

    assert!(rendered.contains("ctypes.c_void_p"), "rendered: {rendered}");
    assert!(rendered.contains("SomeStructure"), "rendered: {rendered}");
    assert!(
        !result
            .log
            .warnings
            .iter()
            .any(|warning| matches!(warning, ConversionWarning::UnsupportedFeature { .. })),
        "mapped types must not warn: {:?}",
        result.log.warnings
    );
}


fn mentions_drop(warnings: &[ConversionWarning]) -> bool {
    warnings.iter().any(|warning| {
        matches!(
            warning,
            ConversionWarning::UnsupportedFeature { resolution, .. }
                if resolution.contains("no native attribute model")
        )
    })
}

#[test]
fn untyped_from_ir_warns_on_dropped_annotations() {
    let mut annotated_struct = neutral_struct();
    annotated_struct.annotations = vec![Annotation::ReprC];

    let mut annotated_function = neutral_function();
    annotated_function.raw_attributes = vec![RawAttribute {
        source: TargetLanguage::Rust,
        text: "inline".to_string(),
    }];

    assert!(mentions_drop(&PythonClass::from_ir(annotated_struct.clone(), None).log.warnings));
    assert!(mentions_drop(&PythonFunction::from_ir(annotated_function.clone(), None).log.warnings));
    assert!(mentions_drop(&JsClass::from_ir(annotated_struct.clone(), None).log.warnings));
    assert!(mentions_drop(&JsFunction::from_ir(annotated_function.clone(), None).log.warnings));
    assert!(mentions_drop(&LuaFunction::from_ir(annotated_function, None).log.warnings));

    // An item with no annotations produces no such warning.
    assert!(!mentions_drop(&PythonClass::from_ir(neutral_struct(), None).log.warnings));
    assert!(!mentions_drop(&LuaFunction::from_ir(neutral_function(), None).log.warnings));
}

#[test]
fn lowers_function_to_lua_snake_case_with_no_types() {
    let function = LuaFunction::from_ir(neutral_function(), None).value;

    assert_eq!(function.name, "compute_total");
    assert_eq!(function.parameters, vec!["item_count".to_string(), "unit_price".to_string()]);

    let backend = LuaBackend::default();
    let mut level = 0;
    let rendered = backend
        .render_function(&function, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    // No types anywhere — bare names and a closing `end`.
    assert_eq!(rendered, "function compute_total(item_count, unit_price)\nend\n");
}
