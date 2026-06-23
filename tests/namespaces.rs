//! Exact-value tests for namespace/module rendering and cross-language namespace conversion.

use langprint::backends::BackendItem;
use langprint::backends::cpp_backend::{
    CppBackend, CppEnum, CppEnumVariant, CppField, CppFunction, CppNamespace, CppStruct, CppStructKind, CppVisibility,
    DocsStyle,
};
use langprint::backends::csharp_backend::{CSharpNamespace, CSharpNamespaceConversionOptions};
use langprint::backends::rust_backend::{
    RustBackend, RustEnum, RustEnumVariant, RustEnumVariantValue, RustFunction, RustModule, RustSelfKind, RustVisibility,
};
use langprint::conversion::ConversionWarning;
use langprint::renderers::NamespaceRenderer;
use langprint::text::{IndentStyle, NewLineStyle};
use langprint::{ConversionConfig, TypeMap};

fn cpp() -> CppBackend {
    CppBackend {
        new_line: NewLineStyle::LF,
        open_brace_on_new_line: true,
        docs_style: DocsStyle::DoubleSlash,
        indent_style: IndentStyle::Spaces,
        indent_size: 4,
    }
}

fn cpp_enum(name: &str) -> CppEnum {
    CppEnum {
        name: name.to_string(),
        variants: vec![CppEnumVariant {
            name: "A".to_string(),
            value: Some("0".to_string()),
            docs: None,
        }],
        is_enum_class: true,
        underlying_type: None,
        docs: None,
    }
}

fn cpp_field(name: &str, ty: &str) -> CppField {
    CppField {
        name: name.to_string(),
        field_type: ty.to_string(),
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
    }
}

fn cpp_struct(name: &str, field_type: &str) -> CppStruct {
    CppStruct {
        struct_kind: CppStructKind::Struct,
        is_final: false,
        alignment: None,
        name: name.to_string(),
        template_params: vec![],
        bases: vec![],
        fields: vec![cpp_field("x", field_type)],
        methods: vec![],
        docs: None,
    }
}

fn cpp_free_function(name: &str) -> CppFunction {
    CppFunction {
        name: name.to_string(),
        parent_name: None,
        visibility: CppVisibility::Public,
        parameters: vec![],
        template_params: vec![],
        return_type: Some("void".to_string()),
        is_static: false,
        is_const: false,
        is_virtual: false,
        is_pure_virtual: false,
        is_inline: false,
        is_noexcept: false,
        is_override: false,
        is_final: false,
        is_friend: false,
        is_deleted: false,
        is_default: false,
        body: Some(vec!["// body".to_string()]),
        docs: None,
    }
}

fn cpp_namespace() -> CppNamespace {
    CppNamespace {
        name: "sdk".to_string(),
        defines: None,
        constants: None,
        enums: Some(vec![cpp_enum("Mode")]),
        structs: None,
        functions: Some(vec![cpp_free_function("init")]),
        namespaces: Some(vec![CppNamespace {
            name: "math".to_string(),
            defines: None,
            constants: None,
            enums: None,
            structs: Some(vec![cpp_struct("Vec2", "float")]),
            functions: None,
            namespaces: None,
        }]),
    }
}

#[test]
fn cpp_namespace_renders() {
    let out = cpp()
        .render_namespace::<&str>(&cpp_namespace(), None, None, None, &mut 0)
        .unwrap();
    assert_eq!(
        out,
        "namespace sdk\n{\n    enum class Mode\n    {\n        A = 0,\n    };\n\n    void init();\n\n    namespace math\n    {\n        struct Vec2\n        {\n        public:\n            float x;\n\n        };\n    }\n}\n"
    );
}

#[test]
fn rust_module_renders() {
    let module = RustModule {
        name: "sdk".to_string(),
        visibility: RustVisibility::Pub,
        defines: None,
        constants: None,
        enums: Some(vec![RustEnum {
            name: "Mode".to_string(),
            visibility: RustVisibility::Pub,
            variants: vec![RustEnumVariant {
                name: "A".to_string(),
                value: RustEnumVariantValue::Discriminant("0".to_string()),
                docs: None,
            }],
            repr: Some("u8".to_string()),
            derives: vec![],
            docs: None,
        }]),
        structs: None,
        functions: Some(vec![RustFunction {
            name: "init".to_string(),
            visibility: RustVisibility::Pub,
            self_kind: RustSelfKind::None,
            parameters: vec![],
            generic_args: vec![],
            return_type: None,
            is_unsafe: false,
            is_async: false,
            is_const: false,
            body: Some(vec!["// body".to_string()]),
            attributes: vec![],
            docs: None,
        }]),
        modules: None,
        docs: None,
    };
    let out = RustBackend::default()
        .render_namespace(&module, None::<&str>, None::<&str>, None, &mut 0)
        .unwrap();
    assert_eq!(
        out,
        "pub mod sdk {\n    #[repr(u8)]\n    pub enum Mode {\n        A = 0,\n    }\n\n    pub fn init() {\n        // body\n    }\n}\n"
    );
}

#[test]
fn cpp_namespace_round_trips_through_ir() {
    let original = cpp_namespace();
    let ir = original.clone().to_ir(None);
    assert!(!ir.log.has_warnings());
    let back = CppNamespace::from_ir(ir.value, None);
    assert!(!back.log.has_warnings());
    let rendered_original = cpp()
        .render_namespace::<&str>(&original, None, None, None, &mut 0)
        .unwrap();
    let rendered_back = cpp()
        .render_namespace::<&str>(&back.value, None, None, None, &mut 0)
        .unwrap();
    assert_eq!(rendered_original, rendered_back);
}

#[test]
fn csharp_namespace_drops_free_functions_with_warning() {
    // A C++ namespace carrying a free function, converted to C#: C# has no namespace-level
    // free functions, so it is dropped with a warning. The namespace name is PascalCased.
    let ir = cpp_namespace().to_ir(None).value;
    let cs = CSharpNamespace::from_ir(ir, None);
    // The free-function drop is reported, then every identifier the config renames across the
    // threaded children: the namespace name, the nested namespace `math`, and the struct field `x`.
    assert_eq!(
        cs.log.warnings,
        vec![
            ConversionWarning::UnsupportedFeature {
                feature: "free functions in namespace `sdk`".to_string(),
                resolution: "C# has no namespace-level free functions; dropped".to_string(),
            },
            ConversionWarning::NamingConventionChanged {
                original: "sdk".to_string(),
                converted: "Sdk".to_string(),
            },
            ConversionWarning::NamingConventionChanged {
                original: "math".to_string(),
                converted: "Math".to_string(),
            },
            ConversionWarning::NamingConventionChanged {
                original: "x".to_string(),
                converted: "X".to_string(),
            },
        ]
    );
    assert_eq!(cs.value.name, "Sdk");
}

#[test]
fn namespace_rename_can_be_disabled() {
    let ir = cpp_namespace().to_ir(None).value;
    let options = CSharpNamespaceConversionOptions {
        config: ConversionConfig::new(TypeMap::builtin(), false),
    };
    let cs = CSharpNamespace::from_ir(ir, Some(&options));
    // With renaming off, the namespace name is verbatim and only the free-function drop is reported.
    assert_eq!(cs.value.name, "sdk");
    assert_eq!(
        cs.log.warnings,
        vec![ConversionWarning::UnsupportedFeature {
            feature: "free functions in namespace `sdk`".to_string(),
            resolution: "C# has no namespace-level free functions; dropped".to_string(),
        }]
    );
}
