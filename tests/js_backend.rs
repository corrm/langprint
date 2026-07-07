//! Tests for the JavaScript backend: exact-output rendering of the form-only model.

use langprint::{
    AVAILABLE_BACKENDS,
    backends::BackendItem,
    backends::js_backend::{
        JsBackend, JsClass, JsEnum, JsEnumMember, JsField, JsFunction, JsFunctionRenderOptions,
        JsParameter,
    },
    conversion::ConversionWarning,
    ir::{
        LanguageFunction, LanguageGenericArgument, LanguageStruct, LanguageStructKind, Visibility,
    },
    renderers::{EnumRenderer, FunctionRenderer},
};

fn warns_generic_arguments(warnings: &[ConversionWarning]) -> bool {
    warnings.iter().any(|warning| {
        matches!(
            warning,
            ConversionWarning::UnsupportedFeature { feature, .. } if feature.contains("generic arguments")
        )
    })
}

#[test]
fn js_is_a_registered_backend() {
    assert!(AVAILABLE_BACKENDS.contains(&"JS"));
}

#[test]
fn renders_function_with_no_body_as_empty_block() {
    let backend = JsBackend::default();
    let function = JsFunction {
        name: "greet".to_string(),
        parameters: vec![JsParameter {
            name: "name".to_string(),
            default: None,
            type_doc: None,
        }],
        return_type: None,
        doc: None,
        is_static: false,
        body: None,
    };

    let mut level = 0;
    let rendered = backend
        .render_function(&function, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    assert_eq!(rendered, "function greet(name) {}\n");
}

#[test]
fn renders_function_with_verbatim_body() {
    let backend = JsBackend::default();
    let function = JsFunction {
        name: "add".to_string(),
        parameters: vec![
            JsParameter {
                name: "a".to_string(),
                default: None,
                type_doc: None,
            },
            JsParameter {
                name: "b".to_string(),
                default: Some("0".to_string()),
                type_doc: None,
            },
        ],
        return_type: None,
        doc: None,
        is_static: false,
        body: Some(vec![
            "const result = a + b;".to_string(),
            "return result;".to_string(),
        ]),
    };

    let mut level = 0;
    let rendered = backend
        .render_function(&function, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    assert_eq!(
        rendered,
        "function add(a, b = 0) {\n  const result = a + b;\n  return result;\n}\n"
    );
}

#[test]
fn renders_jsdoc_when_type_info_present() {
    let backend = JsBackend::default();
    let function = JsFunction {
        name: "add".to_string(),
        parameters: vec![
            JsParameter {
                name: "a".to_string(),
                default: None,
                type_doc: Some("number".to_string()),
            },
            JsParameter {
                name: "b".to_string(),
                default: None,
                type_doc: Some("number".to_string()),
            },
        ],
        return_type: Some("number".to_string()),
        doc: Some("Add two numbers.".to_string()),
        is_static: false,
        body: Some(vec!["return a + b;".to_string()]),
    };

    let mut level = 0;
    let rendered = backend
        .render_function(&function, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    assert_eq!(
        rendered,
        "/**\n * Add two numbers.\n * @param {number} a\n * @param {number} b\n * @returns {number}\n */\nfunction add(a, b) {\n  return a + b;\n}\n"
    );
}

#[test]
fn renders_no_jsdoc_when_type_info_absent() {
    let backend = JsBackend::default();
    let function = JsFunction {
        name: "noop".to_string(),
        parameters: vec![],
        return_type: None,
        doc: None,
        is_static: false,
        body: None,
    };

    let mut level = 0;
    let rendered = backend
        .render_function(&function, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    assert_eq!(rendered, "function noop() {}\n");
    assert!(!rendered.contains("/**"));
}

#[test]
fn renders_class_with_method_and_field() {
    let backend = JsBackend::default();
    let class = JsClass {
        name: "Counter".to_string(),
        extends: None,
        fields: vec![JsField {
            name: "total".to_string(),
            value: "0".to_string(),
            is_static: false,
        }],
        methods: vec![JsFunction {
            name: "increment".to_string(),
            parameters: vec![],
            return_type: None,
            doc: None,
            is_static: false,
            body: Some(vec!["this.total += 1;".to_string()]),
        }],
        doc: None,
    };

    let mut level = 0;
    let rendered = backend
        .render_class(&class, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    assert_eq!(
        rendered,
        "class Counter {\n  total = 0;\n\n  increment() {\n    this.total += 1;\n  }\n}\n"
    );
}

#[test]
fn renders_class_extends() {
    let backend = JsBackend::default();
    let class = JsClass {
        name: "Dog".to_string(),
        extends: Some("Animal".to_string()),
        fields: vec![JsField {
            name: "legs".to_string(),
            value: "4".to_string(),
            is_static: false,
        }],
        methods: vec![JsFunction {
            name: "speak".to_string(),
            parameters: vec![],
            return_type: None,
            doc: None,
            is_static: false,
            body: Some(vec!["return \"woof\";".to_string()]),
        }],
        doc: None,
    };

    let mut level = 0;
    let rendered = backend
        .render_class(&class, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    assert_eq!(
        rendered,
        "class Dog extends Animal {\n  legs = 4;\n\n  speak() {\n    return \"woof\";\n  }\n}\n"
    );
}

#[test]
fn function_from_ir_warns_on_dropped_generics() {
    let function = LanguageFunction {
        name: "identity".to_string(),
        visibility: Visibility::Public,
        parameters: vec![],
        generic_args: vec![LanguageGenericArgument {
            name: "T".to_string(),
            keyword: String::new(),
            default_value: None,
            where_clause: None,
        }],
        return_type: None,
        is_static: false,
        is_abstract: false,
        is_virtual: false,
        is_override: false,
        is_final: false,
        body: None,
        docs: None,
        annotations: vec![],
        raw_attributes: vec![],
    };

    let result = JsFunction::from_ir(function, None);
    assert!(warns_generic_arguments(&result.log.warnings));
}

#[test]
fn class_from_ir_warns_on_dropped_generics() {
    let class = LanguageStruct {
        visibility: Visibility::Public,
        struct_kind: LanguageStructKind::Class,
        is_abstract: false,
        is_final: false,
        name: "Box".to_string(),
        generic_args: vec![LanguageGenericArgument {
            name: "T".to_string(),
            keyword: String::new(),
            default_value: None,
            where_clause: None,
        }],
        bases: vec![],
        fields: vec![],
        methods: vec![],
        docs: None,
        annotations: vec![],
        raw_attributes: vec![],
    };

    let result = JsClass::from_ir(class, None);
    assert!(warns_generic_arguments(&result.log.warnings));
}

#[test]
fn typescript_mode_emits_inline_param_and_return_types() {
    let backend = JsBackend::default();
    let function = JsFunction {
        name: "decode_abi".to_string(),
        parameters: vec![
            JsParameter {
                name: "impl".to_string(),
                default: None,
                type_doc: Some("any".to_string()),
            },
            JsParameter {
                name: "args_ptr".to_string(),
                default: None,
                type_doc: Some("number".to_string()),
            },
        ],
        return_type: Some("number".to_string()),
        doc: None,
        is_static: false,
        body: Some(vec!["return 0;".to_string()]),
    };

    let options = JsFunctionRenderOptions {
        render_jsdoc: false,
        typescript: true,
        verbatim_body: false,
    };
    let mut level = 0;
    let rendered = backend
        .render_function(
            &function,
            None::<&str>,
            None::<&str>,
            Some(&options),
            &mut level,
        )
        .unwrap();

    assert_eq!(
        rendered,
        "function decode_abi(impl: any, args_ptr: number): number {\n  return 0;\n}\n"
    );
}

#[test]
fn typescript_mode_export_via_before_and_typed_default() {
    let backend = JsBackend::default();
    let function = JsFunction {
        name: "make".to_string(),
        parameters: vec![JsParameter {
            name: "count".to_string(),
            default: Some("0".to_string()),
            type_doc: Some("number".to_string()),
        }],
        return_type: Some("void".to_string()),
        doc: None,
        is_static: false,
        body: None,
    };

    let options = JsFunctionRenderOptions {
        render_jsdoc: false,
        typescript: true,
        verbatim_body: false,
    };
    let mut level = 0;
    let rendered = backend
        .render_function(
            &function,
            Some("export "),
            None::<&str>,
            Some(&options),
            &mut level,
        )
        .unwrap();

    // `export` rides the `before` slot; the typed default renders `name: type = value`.
    assert_eq!(
        rendered,
        "export function make(count: number = 0): void {}\n"
    );
}

#[test]
fn typescript_flag_off_keeps_untyped_javascript_signature() {
    let backend = JsBackend::default();
    let function = JsFunction {
        name: "decode_abi".to_string(),
        parameters: vec![JsParameter {
            name: "impl".to_string(),
            default: None,
            type_doc: Some("any".to_string()),
        }],
        return_type: Some("number".to_string()),
        doc: None,
        is_static: false,
        body: Some(vec!["return 0;".to_string()]),
    };

    // Default options (typescript = false): the type strings stay JSDoc-only and
    // the signature is untyped, exactly as before this feature.
    let options = JsFunctionRenderOptions {
        render_jsdoc: false,
        typescript: false,
        verbatim_body: false,
    };
    let mut level = 0;
    let rendered = backend
        .render_function(
            &function,
            None::<&str>,
            None::<&str>,
            Some(&options),
            &mut level,
        )
        .unwrap();

    assert_eq!(rendered, "function decode_abi(impl) {\n  return 0;\n}\n");
}

#[test]
fn renders_ts_const_object_enum() {
    // 4-space indent to match a QuickJS-style consumer.
    let backend = JsBackend {
        indent_size: 4,
        ..JsBackend::default()
    };
    let value = JsEnum {
        name: "LogLevel".to_string(),
        members: vec![
            JsEnumMember {
                name: "Debug".to_string(),
                value: "0".to_string(),
            },
            JsEnumMember {
                name: "Info".to_string(),
                value: "1".to_string(),
            },
        ],
        doc: Some("Enum LogLevel".to_string()),
        export: true,
    };

    let mut level = 0;
    let rendered = backend
        .render_enum::<&str>(&value, None, None, None, &mut level)
        .unwrap();

    assert_eq!(
        rendered,
        "/** Enum LogLevel */\n\
         export const LogLevel = Object.freeze({\n\
         \x20   Debug: 0,\n\
         \x20   Info: 1,\n\
         } as const);\n\
         export type LogLevel = typeof LogLevel[keyof typeof LogLevel];\n"
    );
}
