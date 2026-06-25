//! Lowering tests: a neutral IR sourced from a typed backend lowers INTO idiomatic Python/Lua/JS
//! via each untyped backend's `from_ir`. This is the "from_ir render target" direction — types are
//! re-spelled (Python PEP-484 hints, JS JSDoc, Lua none) and identifiers renamed to convention.
//! The reverse (untyped → IR transpile) is explicitly out of scope and not tested here.

use langprint::backends::js_backend::{JsClass, JsFunction};
use langprint::backends::lua_backend::{LuaBackend, LuaFunction};
use langprint::backends::python_backend::{PythonClass, PythonFunction};
use langprint::backends::BackendItem;
use langprint::ir::{
    LanguageBase, LanguageField, LanguageFunction, LanguageFunctionParameter, LanguageStruct, LanguageStructKind,
    Visibility,
};
use langprint::renderers::FunctionRenderer;

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
