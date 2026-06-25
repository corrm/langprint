//! Tests for the Lua backend: exact-output rendering of the form-only model.

use langprint::{
    AVAILABLE_BACKENDS,
    backends::BackendItem,
    backends::lua_backend::{LuaBackend, LuaField, LuaFunction, LuaModule},
    conversion::ConversionWarning,
    ir::{LanguageConstant, LanguageEnum, LanguageNamespace, Visibility},
    renderers::FunctionRenderer,
};

#[test]
fn lua_is_a_registered_backend() {
    assert!(AVAILABLE_BACKENDS.contains(&"Lua"));
}

#[test]
fn renders_function_with_no_body_as_empty() {
    let backend = LuaBackend::default();
    let function = LuaFunction {
        name: "greet".to_string(),
        parameters: vec!["name".to_string()],
        doc: None,
        body: None,
    };

    let mut level = 0;
    let rendered = backend
        .render_function(&function, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    assert_eq!(rendered, "function greet(name)\nend\n");
}

#[test]
fn renders_function_with_verbatim_body() {
    let backend = LuaBackend::default();
    let function = LuaFunction {
        name: "add".to_string(),
        parameters: vec!["a".to_string(), "b".to_string()],
        doc: None,
        body: Some(vec!["local result = a + b".to_string(), "return result".to_string()]),
    };

    let mut level = 0;
    let rendered = backend
        .render_function(&function, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    assert_eq!(
        rendered,
        "function add(a, b)\n  local result = a + b\n  return result\nend\n"
    );
}

#[test]
fn renders_field_assignment_in_module() {
    let backend = LuaBackend::default();
    let module = LuaModule {
        table_name: "M".to_string(),
        fields: vec![LuaField {
            name: "M.version".to_string(),
            value: "\"1.0\"".to_string(),
        }],
        functions: vec![],
        doc: None,
    };

    let mut level = 0;
    let rendered = backend
        .render_module(&module, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    assert_eq!(rendered, "local M = {}\nM.version = \"1.0\"\n\nreturn M\n");
}

#[test]
fn renders_module_table_with_function_and_field() {
    let backend = LuaBackend::default();
    let module = LuaModule {
        table_name: "M".to_string(),
        fields: vec![LuaField {
            name: "M.version".to_string(),
            value: "\"1.0\"".to_string(),
        }],
        functions: vec![LuaFunction {
            name: "M.greet".to_string(),
            parameters: vec!["name".to_string()],
            doc: Some("Greets a person.".to_string()),
            body: Some(vec!["return \"hi \" .. name".to_string()]),
        }],
        doc: None,
    };

    let mut level = 0;
    let rendered = backend
        .render_module(&module, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();

    assert_eq!(
        rendered,
        "local M = {}\nM.version = \"1.0\"\n\n-- Greets a person.\nfunction M.greet(name)\n  return \"hi \" .. name\nend\n\nreturn M\n"
    );
}

#[test]
fn module_fields_round_trip_through_constants() {
    let module = LuaModule {
        table_name: "M".to_string(),
        fields: vec![LuaField {
            name: "version".to_string(),
            value: "\"1.0\"".to_string(),
        }],
        functions: vec![],
        doc: None,
    };

    let ir = module.clone().to_ir(None);
    assert!(ir.log.warnings.is_empty());
    let constants = ir.value.constants.as_ref().unwrap();
    assert_eq!(constants.len(), 1);
    assert_eq!(constants[0].name, "version");
    assert_eq!(constants[0].value, "\"1.0\"");

    let back = LuaModule::from_ir(ir.value, None);
    assert_eq!(back.value.fields, module.fields);
}

#[test]
fn module_from_ir_warns_on_dropped_namespace_members() {
    let namespace = LanguageNamespace {
        name: "M".to_string(),
        visibility: Visibility::Public,
        defines: None,
        constants: Some(vec![LanguageConstant {
            name: "version".to_string(),
            visibility: Visibility::Public,
            data_type: String::new(),
            value: "\"1.0\"".to_string(),
            docs: None,
        }]),
        enums: Some(vec![LanguageEnum {
            name: "Color".to_string(),
            visibility: Visibility::Public,
            variants: vec![],
            underlying_type: None,
            docs: None,
            annotations: vec![],
            raw_attributes: vec![],
        }]),
        structs: None,
        functions: None,
        namespaces: None,
        docs: None,
    };

    let result = LuaModule::from_ir(namespace, None);
    assert_eq!(result.value.fields.len(), 1);
    assert!(result.log.warnings.iter().any(|warning| matches!(
        warning,
        ConversionWarning::UnsupportedFeature { feature, .. } if feature.contains("nested enums")
    )));
}
