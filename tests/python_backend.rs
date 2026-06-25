//! Tests for the Python backend: exact-output rendering of the form-only model.

use langprint::{
    AVAILABLE_BACKENDS,
    backends::python_backend::{
        PythonBackend, PythonClass, PythonClassField, PythonEnum, PythonEnumMember, PythonFunction, PythonParameter,
        PythonStruct, PythonStructField,
    },
    renderers::{EnumRenderer, FunctionRenderer, StructRenderer},
};

#[test]
fn python_is_a_registered_backend() {
    assert!(AVAILABLE_BACKENDS.contains(&"Python"));
}

#[test]
fn renders_def_with_no_body_as_pass() {
    let backend = PythonBackend::default();
    let function = PythonFunction {
        name: "greet".to_string(),
        parameters: vec![PythonParameter {
            name: "name".to_string(),
            type_hint: Some("str".to_string()),
            default: None,
        }],
        return_type: Some("str".to_string()),
        docstring: None,
        body: None,
    };

    let mut level = 0;
    let rendered = backend
        .render_function(&function, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    assert_eq!(rendered, "def greet(name: str) -> str:\n    pass\n");
}

#[test]
fn renders_def_with_verbatim_body() {
    let backend = PythonBackend::default();
    let function = PythonFunction {
        name: "add".to_string(),
        parameters: vec![
            PythonParameter {
                name: "a".to_string(),
                type_hint: Some("int".to_string()),
                default: None,
            },
            PythonParameter {
                name: "b".to_string(),
                type_hint: Some("int".to_string()),
                default: Some("0".to_string()),
            },
        ],
        return_type: Some("int".to_string()),
        docstring: None,
        body: Some(vec!["result = a + b".to_string(), "return result".to_string()]),
    };

    let mut level = 0;
    let rendered = backend
        .render_function(&function, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    assert_eq!(
        rendered,
        "def add(a: int, b: int = 0) -> int:\n    result = a + b\n    return result\n"
    );
}

#[test]
fn renders_class_with_method() {
    let backend = PythonBackend::default();
    let class = PythonClass {
        name: "Counter".to_string(),
        bases: vec![],
        fields: vec![PythonClassField {
            name: "total".to_string(),
            value: "0".to_string(),
        }],
        methods: vec![PythonFunction {
            name: "increment".to_string(),
            parameters: vec![PythonParameter {
                name: "self".to_string(),
                type_hint: None,
                default: None,
            }],
            return_type: None,
            docstring: None,
            body: Some(vec!["self.total += 1".to_string()]),
        }],
        docstring: None,
    };

    let mut level = 0;
    let rendered = backend
        .render_class(&class, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    assert_eq!(
        rendered,
        "class Counter:\n    total = 0\n\n    def increment(self):\n        self.total += 1\n"
    );
}

#[test]
fn renders_ctypes_structure() {
    let backend = PythonBackend::default();
    let value = PythonStruct {
        name: "Point".to_string(),
        fields: vec![
            PythonStructField {
                name: "x".to_string(),
                ctype: "ctypes.c_int32".to_string(),
            },
            PythonStructField {
                name: "y".to_string(),
                ctype: "ctypes.c_int32".to_string(),
            },
        ],
        docstring: None,
    };

    let mut level = 0;
    let rendered = backend
        .render_struct(&value, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    assert_eq!(
        rendered,
        "class Point(ctypes.Structure):\n    _fields_ = [\n        (\"x\", ctypes.c_int32),\n        (\"y\", ctypes.c_int32),\n    ]\n"
    );
}

#[test]
fn renders_int_enum() {
    let backend = PythonBackend::default();
    let value = PythonEnum {
        name: "Color".to_string(),
        members: vec![
            PythonEnumMember {
                name: "RED".to_string(),
                value: "0".to_string(),
            },
            PythonEnumMember {
                name: "GREEN".to_string(),
                value: "1".to_string(),
            },
            PythonEnumMember {
                name: "BLUE".to_string(),
                value: "2".to_string(),
            },
        ],
        docstring: None,
    };

    let mut level = 0;
    let rendered = backend
        .render_enum(&value, None::<&str>, None::<&str>, None, None, &mut level)
        .unwrap();

    assert_eq!(
        rendered,
        "class Color(enum.IntEnum):\n    RED = 0\n    GREEN = 1\n    BLUE = 2\n"
    );
}

#[test]
fn renders_class_with_base_and_docstring() {
    let backend = PythonBackend::default();
    let class = PythonClass {
        name: "Animal".to_string(),
        bases: vec!["Base".to_string()],
        fields: vec![],
        methods: vec![],
        docstring: Some("An animal.".to_string()),
    };

    let mut level = 0;
    let rendered = backend
        .render_class(&class, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    assert_eq!(rendered, "class Animal(Base):\n    \"\"\"An animal.\"\"\"\n");
}

#[test]
fn renders_multiline_docstring_with_indented_continuation() {
    let backend = PythonBackend::default();
    let class = PythonClass {
        name: "Animal".to_string(),
        bases: vec![],
        fields: vec![],
        methods: vec![],
        docstring: Some("Line1\nLine2".to_string()),
    };

    let mut level = 0;
    let rendered = backend
        .render_class(&class, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    assert_eq!(rendered, "class Animal:\n    \"\"\"Line1\n    Line2\"\"\"\n");
}
