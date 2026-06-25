//! Tests for the Rust backend: exact-output rendering and IR round-trip/warning behavior.

use langprint::{
    AVAILABLE_BACKENDS,
    backends::{
        BackendItem,
        rust_backend::{
            RustBackend, RustConstant, RustDefinition, RustEnum, RustEnumVariant, RustEnumVariantValue, RustField,
            RustFunction, RustParameter, RustSelfKind, RustStruct, RustVisibility,
        },
    },
    conversion::ConversionWarning,
    ir::Visibility,
    renderers::{ConstantRenderer, DefinitionRenderer, EnumRenderer, FunctionRenderer, StructRenderer},
};

#[test]
fn rust_is_a_registered_backend() {
    assert!(AVAILABLE_BACKENDS.contains(&"Rust"));
}

#[test]
fn renders_constant_with_docs() {
    let backend = RustBackend::default();
    let constant = RustConstant {
        name: "MAX".to_string(),
        visibility: RustVisibility::Pub,
        data_type: "u32".to_string(),
        value: "100".to_string(),
        is_static: false,
        docs: Some(vec!["The maximum value.".to_string()]),
    };

    let mut level = 0;
    let rendered = backend
        .render_constant(&constant, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    assert_eq!(rendered, "/// The maximum value.\npub const MAX: u32 = 100;\n");
}

#[test]
fn renders_definition_with_and_without_value() {
    let backend = RustBackend::default();

    let with_value = RustDefinition {
        name: "VERSION".to_string(),
        value: Some("1".to_string()),
        docs: None,
    };
    let without_value = RustDefinition {
        name: "FLAG".to_string(),
        value: None,
        docs: None,
    };

    let mut level = 0;
    assert_eq!(
        backend
            .render_definition(&with_value, None::<&str>, None::<&str>, None, &mut level)
            .unwrap(),
        "pub const VERSION: i64 = 1;\n"
    );
    assert_eq!(
        backend
            .render_definition(&without_value, None::<&str>, None::<&str>, None, &mut level)
            .unwrap(),
        "pub const FLAG: () = ();\n"
    );
}

#[test]
fn renders_data_carrying_enum() {
    let backend = RustBackend::default();
    let value = RustEnum {
        name: "Shape".to_string(),
        visibility: RustVisibility::Pub,
        variants: vec![
            RustEnumVariant {
                name: "None".to_string(),
                value: RustEnumVariantValue::Unit,
                docs: None,
            },
            RustEnumVariant {
                name: "Circle".to_string(),
                value: RustEnumVariantValue::Tuple(vec!["f32".to_string()]),
                docs: None,
            },
            RustEnumVariant {
                name: "Rect".to_string(),
                value: RustEnumVariantValue::Struct(vec![
                    ("w".to_string(), "f32".to_string()),
                    ("h".to_string(), "f32".to_string()),
                ]),
                docs: None,
            },
            RustEnumVariant {
                name: "Marker".to_string(),
                value: RustEnumVariantValue::Discriminant("5".to_string()),
                docs: None,
            },
        ],
        repr: Some("u8".to_string()),
        derives: vec!["Debug".to_string(), "Clone".to_string()],
        docs: None,
    };

    let mut level = 0;
    let rendered = backend
        .render_enum(&value, None::<&str>, None::<&str>, None, None, &mut level)
        .unwrap();

    assert_eq!(
        rendered,
        "#[derive(Debug, Clone)]\n#[repr(u8)]\npub enum Shape {\n    None,\n    Circle(f32),\n    Rect { w: f32, h: f32 },\n    Marker = 5,\n}\n"
    );
}

#[test]
fn renders_free_function() {
    let backend = RustBackend::default();
    let function = RustFunction {
        name: "add".to_string(),
        visibility: RustVisibility::Pub,
        self_kind: RustSelfKind::None,
        parameters: vec![
            RustParameter {
                name: "a".to_string(),
                param_type: "i32".to_string(),
            },
            RustParameter {
                name: "b".to_string(),
                param_type: "i32".to_string(),
            },
        ],
        generic_args: vec![],
        return_type: Some("i32".to_string()),
        is_unsafe: false,
        is_async: false,
        is_const: false,
        abi: None,
        body: Some(vec!["a + b".to_string()]),
        attributes: vec![],
        docs: None,
    };

    let mut level = 0;
    let rendered = backend
        .render_function(&function, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    assert_eq!(rendered, "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n");
}

#[test]
fn renders_unsafe_extern_c_function() {
    let backend = RustBackend::default();
    let function = RustFunction {
        name: "polyplug_init".to_string(),
        visibility: RustVisibility::Pub,
        self_kind: RustSelfKind::None,
        parameters: vec![],
        generic_args: vec![],
        return_type: None,
        is_unsafe: true,
        is_async: false,
        is_const: false,
        abi: Some("C".to_string()),
        body: Some(vec![]),
        attributes: vec![],
        docs: None,
    };

    let mut level = 0;
    let rendered = backend
        .render_function(&function, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    assert!(rendered.starts_with("pub unsafe extern \"C\" fn polyplug_init("));
}

#[test]
fn non_extern_function_omits_extern_specifier() {
    let backend = RustBackend::default();
    let function = RustFunction {
        name: "plain".to_string(),
        visibility: RustVisibility::Pub,
        self_kind: RustSelfKind::None,
        parameters: vec![],
        generic_args: vec![],
        return_type: None,
        is_unsafe: false,
        is_async: false,
        is_const: false,
        abi: None,
        body: Some(vec![]),
        attributes: vec![],
        docs: None,
    };

    let mut level = 0;
    let rendered = backend
        .render_function(&function, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    assert!(!rendered.contains("extern"));
    assert!(rendered.starts_with("pub fn plain("));
}

#[test]
fn renders_struct_with_impl_block() {
    let backend = RustBackend::default();
    let value = RustStruct {
        name: "Player".to_string(),
        visibility: RustVisibility::Pub,
        generic_args: vec![],
        fields: vec![
            RustField {
                name: "health".to_string(),
                field_type: "f32".to_string(),
                visibility: RustVisibility::Pub,
                attributes: vec![],
                docs: None,
            },
            RustField {
                name: "name".to_string(),
                field_type: "String".to_string(),
                visibility: RustVisibility::Private,
                attributes: vec![],
                docs: None,
            },
        ],
        methods: vec![RustFunction {
            name: "heal".to_string(),
            visibility: RustVisibility::Pub,
            self_kind: RustSelfKind::RefMut,
            parameters: vec![RustParameter {
                name: "amount".to_string(),
                param_type: "f32".to_string(),
            }],
            generic_args: vec![],
            return_type: None,
            is_unsafe: false,
            is_async: false,
            is_const: false,
            abi: None,
            body: Some(vec!["self.health += amount;".to_string()]),
            attributes: vec![],
            docs: None,
        }],
        derives: vec!["Debug".to_string()],
        attributes: vec!["repr(C)".to_string()],
        is_tuple: false,
        docs: None,
    };

    let mut level = 0;
    let rendered = backend
        .render_struct(&value, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    assert_eq!(
        rendered,
        "#[derive(Debug)]\n#[repr(C)]\npub struct Player {\n    pub health: f32,\n    name: String,\n}\n\nimpl Player {\n    pub fn heal(&mut self, amount: f32) {\n        self.health += amount;\n    }\n}\n"
    );
}

#[test]
fn enum_variant_payloads_round_trip_through_ir() {
    let original = RustEnum {
        name: "Shape".to_string(),
        visibility: RustVisibility::Pub,
        variants: vec![
            RustEnumVariant {
                name: "Unit".to_string(),
                value: RustEnumVariantValue::Unit,
                docs: None,
            },
            RustEnumVariant {
                name: "Tup".to_string(),
                value: RustEnumVariantValue::Tuple(vec!["i32".to_string(), "u8".to_string()]),
                docs: None,
            },
            RustEnumVariant {
                name: "Strct".to_string(),
                value: RustEnumVariantValue::Struct(vec![("x".to_string(), "f32".to_string())]),
                docs: None,
            },
        ],
        repr: None,
        derives: vec![],
        docs: None,
    };

    let ir = original.clone().to_ir(None).value;
    let round_tripped = RustEnum::from_ir(ir, None).value;

    let values: Vec<RustEnumVariantValue> = round_tripped.variants.iter().map(|v| v.value.clone()).collect();
    assert_eq!(
        values,
        vec![
            RustEnumVariantValue::Unit,
            RustEnumVariantValue::Tuple(vec!["i32".to_string(), "u8".to_string()]),
            RustEnumVariantValue::Struct(vec![("x".to_string(), "f32".to_string())]),
        ]
    );
}

#[test]
fn struct_to_ir_warns_on_rust_only_features() {
    let value = RustStruct {
        name: "Foo".to_string(),
        visibility: RustVisibility::Pub,
        generic_args: vec![],
        fields: vec![RustField {
            name: "x".to_string(),
            field_type: "i32".to_string(),
            visibility: RustVisibility::Pub,
            attributes: vec![],
            docs: None,
        }],
        methods: vec![],
        derives: vec!["Debug".to_string()],
        attributes: vec!["repr(C)".to_string()],
        is_tuple: false,
        docs: None,
    };

    let result = value.to_ir(None);
    // One warning for the derive, one for the attribute.
    assert_eq!(result.log.warnings.len(), 2);
    assert!(
        result
            .log
            .warnings
            .iter()
            .all(|w| matches!(w, ConversionWarning::UnsupportedFeature { .. }))
    );

    // Common fields survive the projection.
    assert_eq!(result.value.name, "Foo");
    assert_eq!(result.value.fields.len(), 1);
    assert_eq!(result.value.fields[0].field_type, "i32");
}

#[test]
fn from_ir_warns_on_protected_visibility() {
    let result = RustVisibility::from_ir(Visibility::Protected, None);
    assert_eq!(result.value, RustVisibility::Pub);
    assert_eq!(result.log.warnings.len(), 1);
    assert!(matches!(
        result.log.warnings[0],
        ConversionWarning::VisibilityApproximated { .. }
    ));
}

#[test]
fn parameter_default_value_is_dropped_with_warning() {
    use langprint::ir::LanguageFunctionParameter;

    let param = LanguageFunctionParameter {
        name: "count".to_string(),
        param_type: "i32".to_string(),
        default_value: Some("0".to_string()),
    };

    let result = RustParameter::from_ir(param, None);
    assert_eq!(result.value.name, "count");
    assert_eq!(result.log.warnings.len(), 1);
    assert!(matches!(
        result.log.warnings[0],
        ConversionWarning::UnsupportedFeature { .. }
    ));
}
