//! Exact-output render tests for the C++ backend.

use langprint::backends::cpp_backend::{
    CppBackend, CppBase, CppConstant, CppEnum, CppEnumVariant, CppField, CppFunction,
    CppFunctionRenderOptions, CppParameter, CppStruct, CppStructKind, CppVisibility, DocsStyle,
};
use langprint::renderers::{ConstantRenderer, EnumRenderer, FunctionRenderer, StructRenderer};
use langprint::text::{IndentStyle, NewLineStyle};

/// A deterministic, LF/4-space backend so rendered output is stable to compare.
fn backend() -> CppBackend {
    CppBackend {
        new_line: NewLineStyle::LF,
        open_brace_on_new_line: true,
        docs_style: DocsStyle::DoubleSlash,
        indent_style: IndentStyle::Spaces,
        indent_size: 4,
        space_before_enum_base: false,
    }
}

fn plain_function(name: &str) -> CppFunction {
    CppFunction {
        name: name.to_string(),
        parent_name: None,
        visibility: CppVisibility::Public,
        parameters: Vec::new(),
        template_params: Vec::new(),
        return_type: Some("void".to_string()),
        is_static: false,
        is_const: false,
        is_virtual: false,
        is_pure_virtual: false,
        is_inline: false,
        is_noexcept: false,
        is_extern_c: false,
        is_override: false,
        is_final: false,
        is_friend: false,
        is_deleted: false,
        is_default: false,
        body: None,
        docs: None,
    }
}

#[test]
fn renders_constant() {
    let be = backend();
    let mut indent = 0;
    let constant = CppConstant {
        name: "MAX".to_string(),
        visibility: CppVisibility::Default,
        data_type: "int".to_string(),
        value: "10".to_string(),
        docs: None,
    };
    let rendered = be
        .render_constant::<&str>(&constant, None, None, None, &mut indent)
        .unwrap();
    assert_eq!(rendered, "const int MAX = 10;\n");
}

#[test]
fn renders_scoped_enum() {
    let be = backend();
    let mut indent = 0;
    let cpp_enum = CppEnum {
        name: "Color".to_string(),
        variants: vec![
            CppEnumVariant {
                name: "Red".to_string(),
                value: Some("0".to_string()),
                docs: None,
            },
            CppEnumVariant {
                name: "Green".to_string(),
                value: Some("1".to_string()),
                docs: None,
            },
        ],
        is_enum_class: true,
        underlying_type: Some("uint8_t".to_string()),
        docs: None,
    };
    let rendered = be
        .render_enum::<&str>(&cpp_enum, None, None, None, &mut indent)
        .unwrap();
    assert_eq!(
        rendered,
        "enum class Color: uint8_t\n{\n    Red = 0,\n    Green = 1,\n};\n"
    );

    // Opt-in spacing: `Name : type`, matching C++ inheritance-list style.
    let mut spaced = backend();
    spaced.space_before_enum_base = true;
    let mut indent = 0;
    let rendered_spaced = spaced
        .render_enum::<&str>(&cpp_enum, None, None, None, &mut indent)
        .unwrap();
    assert_eq!(
        rendered_spaced,
        "enum class Color : uint8_t\n{\n    Red = 0,\n    Green = 1,\n};\n"
    );
}

#[test]
fn renders_unscoped_enum() {
    let be = backend();
    let mut indent = 0;
    let cpp_enum = CppEnum {
        name: "Color".to_string(),
        variants: vec![
            CppEnumVariant {
                name: "Red".to_string(),
                value: Some("0".to_string()),
                docs: None,
            },
            CppEnumVariant {
                name: "Green".to_string(),
                value: Some("1".to_string()),
                docs: None,
            },
        ],
        is_enum_class: false,
        underlying_type: Some("uint8_t".to_string()),
        docs: None,
    };
    let rendered = be
        .render_enum::<&str>(&cpp_enum, None, None, None, &mut indent)
        .unwrap();
    assert_eq!(
        rendered,
        "enum Color: uint8_t\n{\n    Red = 0,\n    Green = 1,\n};\n"
    );
}

/// A no-value enumerator must still be indented and comma-terminated — otherwise the emitted enum
/// body is invalid C++ (`enum { North South };` won't compile). Regression guard.
#[test]
fn renders_enum_with_no_value_variants() {
    let be = backend();
    let mut indent = 0;
    let cpp_enum = CppEnum {
        name: "Direction".to_string(),
        variants: vec![
            CppEnumVariant {
                name: "North".to_string(),
                value: None,
                docs: None,
            },
            CppEnumVariant {
                name: "South".to_string(),
                value: None,
                docs: None,
            },
        ],
        is_enum_class: true,
        underlying_type: None,
        docs: None,
    };
    let rendered = be
        .render_enum::<&str>(&cpp_enum, None, None, None, &mut indent)
        .unwrap();
    assert_eq!(
        rendered,
        "enum class Direction\n{\n    North,\n    South,\n};\n"
    );
}

#[test]
fn renders_function_declaration() {
    let be = backend();
    let mut indent = 0;
    let mut decl = plain_function("init");
    decl.parameters = vec![CppParameter {
        name: "count".to_string(),
        param_type: "int".to_string(),
        default_value: None,
    }];
    let rendered = be
        .render_function::<&str>(&decl, None, None, None, &mut indent)
        .unwrap();
    assert_eq!(rendered, "void init(int count);");
}

#[test]
fn renders_function_definition_with_parent_and_body() {
    let be = backend();
    let mut indent = 0;
    let mut def = plain_function("init");
    def.parent_name = Some("Engine".to_string());
    def.body = Some(vec!["return;".to_string()]);
    let opts = CppFunctionRenderOptions {
        render_definition: true,
        force_render_body: true,
        ..CppFunctionRenderOptions::DEFAULT
    };
    let rendered = be
        .render_function::<&str>(&def, None, None, Some(&opts), &mut indent)
        .unwrap();
    assert_eq!(rendered, "void Engine::init()\n{\n    return;\n}");
}

#[test]
fn renders_struct_with_base_field_and_method() {
    let be = backend();
    let mut indent = 0;
    let cpp_struct = CppStruct {
        struct_kind: CppStructKind::Class,
        is_final: false,
        alignment: None,
        is_packed: false,
        name: "Player".to_string(),
        template_params: Vec::new(),
        bases: vec![CppBase {
            name: "Entity".to_string(),
            visibility: CppVisibility::Public,
        }],
        fields: vec![CppField {
            name: "health".to_string(),
            field_type: "int".to_string(),
            visibility: CppVisibility::Public,
            array_size: None,
            bit_field_size: None,
            alignment: None,
            is_static: false,
            is_const: false,
            is_inline: false,
            initialization_value: None,
            inline_comment: None,
            docs: None,
        }],
        methods: vec![plain_function("tick")],
        docs: None,
    };
    let rendered = be
        .render_struct::<&str>(&cpp_struct, None, None, None, &mut indent)
        .unwrap();
    assert_eq!(
        rendered,
        "class Player : public Entity\n{\npublic:\n    int health;\n\npublic:\n    void tick();\n};\n"
    );
}
