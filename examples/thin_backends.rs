//! Demonstrates the thin, render-only backends (Python, Lua, JS): each native model is built
//! directly and rendered, with its `body` slot filled by the caller. The final section shows a
//! custom map — a ctypes `TypeMap` plus a `type_override` for a non-primitive handle type — driving
//! the IR lowering path.
//!
//! Run with: `cargo run -p langprint --example thin_backends`

use std::sync::Arc;

use langprint::backends::BackendItem;
use langprint::backends::js_backend::{JsBackend, JsClass, JsField, JsFunction};
use langprint::backends::lua_backend::{LuaBackend, LuaFunction};
use langprint::backends::python_backend::{
    ctypes_type_map, PythonBackend, PythonFunction, PythonParameter, PythonStruct, PythonStructConversionOptions,
};
use langprint::convert::ConversionConfig;
use langprint::ir::{LanguageField, LanguageStruct, LanguageStructKind, Visibility};
use langprint::renderers::{FunctionRenderer, StructRenderer};
use langprint::type_map::TargetLanguage;

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

fn main() {
    // ── Python: a `def` with a filled body, rendered directly (no IR) ──
    let greet = PythonFunction {
        name: "greet".to_string(),
        parameters: vec![PythonParameter {
            name: "name".to_string(),
            type_hint: Some("str".to_string()),
            default: None,
        }],
        return_type: Some("str".to_string()),
        docstring: Some("Build a greeting.".to_string()),
        body: Some(vec!["return f\"hello {name}\"".to_string()]),
    };
    let python_src = PythonBackend::default()
        .render_function(&greet, None::<&str>, None::<&str>, None, &mut 0)
        .expect("render Python def");
    println!("== Python (def) ==\n{python_src}");

    // ── Lua: a module function `M.greet` with a filled body ──
    let lua_greet = LuaFunction {
        name: "M.greet".to_string(),
        parameters: vec!["name".to_string()],
        doc: Some("Greet by name.".to_string()),
        body: Some(vec!["return \"hello \" .. name".to_string()]),
    };
    let lua_src = LuaBackend::default()
        .render_function(&lua_greet, None::<&str>, None::<&str>, None, &mut 0)
        .expect("render Lua function");
    println!("== Lua (module function) ==\n{lua_src}");

    // ── JS: a class whose method body is rendered verbatim ──
    let counter = JsClass {
        name: "Counter".to_string(),
        extends: None,
        fields: vec![JsField {
            name: "count".to_string(),
            value: "0".to_string(),
            is_static: false,
        }],
        methods: vec![JsFunction {
            name: "increment".to_string(),
            parameters: vec![],
            return_type: Some("number".to_string()),
            doc: None,
            is_static: false,
            body: Some(vec!["this.count += 1;".to_string(), "return this.count;".to_string()]),
        }],
        doc: None,
    };
    let js_src = JsBackend::default()
        .render_class::<&str>(&counter, None, None, None, &mut 0)
        .expect("render JS class");
    println!("== JS (class) ==\n{js_src}");

    // ── Custom map: ctypes TypeMap + a type_override for a non-primitive handle ──
    // Primitives (`u32` → `ctypes.c_uint32`) come from the ctypes map; `MyHandle` is not a
    // primitive, so a `type_override` re-spells it to `ctypes.c_void_p`.
    let mut config = ConversionConfig::new(ctypes_type_map(), false);
    config.type_override = Some(Arc::new(|spelling: &str, language: TargetLanguage| {
        (language == TargetLanguage::Python && spelling == "MyHandle").then(|| "ctypes.c_void_p".to_string())
    }));

    let handle_struct = LanguageStruct {
        visibility: Visibility::Public,
        struct_kind: LanguageStructKind::Struct,
        is_abstract: false,
        is_final: false,
        name: "Resource".to_string(),
        generic_args: Vec::new(),
        bases: Vec::new(),
        fields: vec![ir_field("handle", "MyHandle"), ir_field("id", "u32")],
        methods: Vec::new(),
        docs: None,
        annotations: Vec::new(),
        raw_attributes: Vec::new(),
    };

    let options = PythonStructConversionOptions { config };
    let resource = PythonStruct::from_ir(handle_struct, Some(&options)).value;
    let resource_src = PythonBackend::default()
        .render_struct(&resource, None::<&str>, None::<&str>, None, &mut 0)
        .expect("render ctypes struct");
    println!("== Python (ctypes struct via custom TypeMap + type_override) ==\n{resource_src}");
}
