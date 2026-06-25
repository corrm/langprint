//! Tests for the JavaScript backend: exact-output rendering of the form-only model.

use langprint::{
    AVAILABLE_BACKENDS,
    backends::js_backend::{JsBackend, JsClass, JsField, JsFunction, JsParameter},
    renderers::FunctionRenderer,
};

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
        body: Some(vec!["const result = a + b;".to_string(), "return result;".to_string()]),
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
