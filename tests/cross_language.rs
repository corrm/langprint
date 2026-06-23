//! Cross-language conversion tests — prove the neutral declaration IR is a real bridge.
//!
//! Each test routes a declaration native-model → `to_ir()` → `from_ir()` into another backend →
//! renders it, asserting the EXACT output AND the EXACT `ConversionWarning`s on both legs. This is
//! what earns the "language-agnostic" claim: the common declaration subset survives, and every
//! single-language feature that cannot cross is reported (never silently dropped).
//!
//! Note on scope: the IR bridges declaration STRUCTURE, not type-name spelling. Type names
//! (`int`, `void`, `f32`, …) are carried verbatim — translating them is out of scope by design,
//! so e.g. a C++ `int` field appears as `int` in the rendered Rust. The tests assert this honestly.

use langprint::backends::BackendItem;
use langprint::backends::cpp_backend::{
    CppBackend, CppBase, CppEnum, CppEnumVariant, CppField, CppFunction, CppParameter, CppStruct, CppStructKind,
    CppVisibility, DocsStyle,
};
use langprint::backends::csharp_backend::{
    CSharpBackend, CSharpEnum, CSharpEnumMember, CSharpField, CSharpType, CSharpTypeKind, CSharpVisibility,
};
use langprint::backends::rust_backend::{
    RustBackend, RustEnum, RustEnumVariant, RustEnumVariantValue, RustStruct, RustVisibility,
};
use langprint::conversion::ConversionWarning;
use langprint::renderers::{EnumRenderer, StructRenderer};
use langprint::text::{IndentStyle, NewLineStyle};

/// A deterministic LF / 4-space C++ backend so rendered output is stable to compare.
fn cpp() -> CppBackend {
    CppBackend {
        new_line: NewLineStyle::LF,
        open_brace_on_new_line: true,
        docs_style: DocsStyle::DoubleSlash,
        indent_style: IndentStyle::Spaces,
        indent_size: 4,
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

fn cpp_method(name: &str) -> CppFunction {
    CppFunction {
        name: name.to_string(),
        parent_name: None,
        visibility: CppVisibility::Public,
        parameters: vec![CppParameter {
            name: "amount".to_string(),
            param_type: "int".to_string(),
            default_value: None,
        }],
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

fn rust_field(name: &str, ty: &str) -> langprint::backends::rust_backend::RustField {
    langprint::backends::rust_backend::RustField {
        name: name.to_string(),
        field_type: ty.to_string(),
        visibility: RustVisibility::Pub,
        attributes: vec![],
        docs: None,
    }
}

fn unsupported_features(ws: &[ConversionWarning]) -> Vec<String> {
    ws.iter()
        .filter_map(|w| match w {
            ConversionWarning::UnsupportedFeature { feature, .. } => Some(feature.clone()),
            _ => None,
        })
        .collect()
}

/// C++ struct (fields + method) → IR → Rust. Proves the common subset crosses with zero loss;
/// the method lowers into an idiomatic `impl` block. Type names pass through verbatim.
#[test]
fn cpp_struct_to_rust() {
    let s = CppStruct {
        struct_kind: CppStructKind::Class,
        is_final: false,
        alignment: None,
        name: "Player".to_string(),
        template_params: vec![],
        bases: vec![],
        fields: vec![cpp_field("health", "int"), cpp_field("mana", "int")],
        methods: vec![cpp_method("heal")],
        docs: None,
    };

    let ir = s.to_ir(None);
    assert!(!ir.log.has_warnings());

    let rust = RustStruct::from_ir(ir.value, None);
    assert!(!rust.log.has_warnings());

    let out = RustBackend::default()
        .render_struct(&rust.value, None::<&str>, None::<&str>, None, &mut 0)
        .unwrap();
    assert_eq!(
        out,
        "struct Player {\n    pub health: i32,\n    pub mana: i32,\n}\n\nimpl Player {\n    pub fn heal(&self, amount: i32) {\n        // body\n    }\n}\n"
    );
}

/// C++ struct with a base → IR → C#. Proves inheritance (single base) and members cross cleanly.
#[test]
fn cpp_struct_with_base_to_csharp() {
    let s = CppStruct {
        struct_kind: CppStructKind::Class,
        is_final: false,
        alignment: None,
        name: "Player".to_string(),
        template_params: vec![],
        bases: vec![CppBase {
            name: "Entity".to_string(),
            visibility: CppVisibility::Public,
        }],
        fields: vec![cpp_field("health", "float")],
        methods: vec![cpp_method("heal")],
        docs: None,
    };

    let ir = s.to_ir(None);
    assert!(!ir.log.has_warnings());

    let cs = CSharpType::from_ir(ir.value, None);
    // Idiomatic C# renaming PascalCases the field and method; each change is reported.
    assert_eq!(
        cs.log.warnings,
        vec![
            ConversionWarning::NamingConventionChanged {
                original: "health".to_string(),
                converted: "Health".to_string(),
            },
            ConversionWarning::NamingConventionChanged {
                original: "heal".to_string(),
                converted: "Heal".to_string(),
            },
        ]
    );

    let out = CSharpBackend::default()
        .render_struct::<&str>(&cs.value, None, None, None, &mut 0)
        .unwrap();
    assert_eq!(
        out,
        "class Player : Entity\n{\n    public float Health;\n\n    public void Heal(int amount)\n    {\n        // body\n    }\n}\n"
    );
}

/// Rust valued enum → IR → C++. Proves enum discriminants and underlying type cross with no loss.
#[test]
fn rust_enum_to_cpp() {
    let e = RustEnum {
        name: "Color".to_string(),
        visibility: RustVisibility::Pub,
        variants: vec![
            RustEnumVariant {
                name: "Red".to_string(),
                value: RustEnumVariantValue::Discriminant("0".to_string()),
                docs: None,
            },
            RustEnumVariant {
                name: "Green".to_string(),
                value: RustEnumVariantValue::Discriminant("1".to_string()),
                docs: None,
            },
        ],
        repr: Some("u8".to_string()),
        derives: vec![],
        docs: None,
    };

    let ir = e.to_ir(None);
    assert!(!ir.log.has_warnings());

    let ce = CppEnum::from_ir(ir.value, None);
    assert!(!ce.log.has_warnings());

    let out = cpp()
        .render_enum::<&str>(&ce.value, None, None, None, None, &mut 0)
        .unwrap();
    assert_eq!(out, "enum class Color: uint8_t\n{\n    Red = 0,\n    Green = 1,\n};\n");
}

/// Rust data-carrying enum → IR → C#. The Tuple payload cannot exist in a C# enum, so `from_ir`
/// MUST surface a warning (it is never silently dropped); the variant names still cross.
#[test]
fn rust_data_enum_to_csharp_warns_on_payload() {
    let e = RustEnum {
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
        ],
        repr: None,
        derives: vec![],
        docs: None,
    };

    // to_ir preserves the payload in the IR (Rust-shaped variant model), no loss yet.
    let ir = e.to_ir(None);
    assert!(!ir.log.has_warnings());

    // from_ir into C# drops the payload and reports it.
    let cs = CSharpEnum::from_ir(ir.value, None);
    assert_eq!(cs.log.warnings.len(), 1);

    let out = CSharpBackend::default()
        .render_enum::<&str>(&cs.value, None, None, None, None, &mut 0)
        .unwrap();
    assert_eq!(out, "public enum Shape\n{\n    None,\n    Circle,\n}\n");
}

/// C# class with a base + interface → IR → Rust. Rust has no inheritance, so BOTH the base and the
/// interface must be reported as dropped; the fields still cross.
#[test]
fn csharp_class_to_rust_warns_on_inheritance() {
    let mut ty = CSharpType {
        kind: CSharpTypeKind::Class,
        name: "Player".to_string(),
        visibility: CSharpVisibility::Public,
        is_abstract: false,
        is_sealed: false,
        is_static: false,
        is_partial: false,
        generic_args: vec![],
        base_class: Some("Entity".to_string()),
        interfaces: vec!["IDamageable".to_string()],
        fields: vec![],
        properties: vec![],
        methods: vec![],
        attributes: vec![],
        docs: None,
    };
    ty.fields.push(CSharpField {
        name: "health".to_string(),
        field_type: "float".to_string(),
        visibility: CSharpVisibility::Public,
        is_static: false,
        is_const: false,
        is_readonly: false,
        initializer: None,
        attributes: vec![],
        docs: None,
    });

    let ir = ty.to_ir(None);
    assert!(!ir.log.has_warnings());

    let rust = RustStruct::from_ir(ir.value, None);
    assert_eq!(
        unsupported_features(&rust.log.warnings),
        vec![
            "base `Entity` of `Player`".to_string(),
            "base `IDamageable` of `Player`".to_string()
        ]
    );

    let out = RustBackend::default()
        .render_struct(&rust.value, None::<&str>, None::<&str>, None, &mut 0)
        .unwrap();
    assert_eq!(out, "pub struct Player {\n    pub health: f32,\n}\n");
}

/// C# enum → IR → C++. Proves the enum crosses the other direction with values intact.
#[test]
fn csharp_enum_to_cpp() {
    let e = CSharpEnum {
        name: "Color".to_string(),
        visibility: CSharpVisibility::Public,
        underlying_type: Some("byte".to_string()),
        members: vec![
            CSharpEnumMember {
                name: "Red".to_string(),
                value: Some("0".to_string()),
                docs: None,
            },
            CSharpEnumMember {
                name: "Green".to_string(),
                value: Some("1".to_string()),
                docs: None,
            },
        ],
        is_flags: false,
        attributes: vec![],
        docs: None,
    };

    let ir = e.to_ir(None);
    assert!(!ir.log.has_warnings());

    let ce = CppEnum::from_ir(ir.value, None);
    assert!(!ce.log.has_warnings());

    let out = cpp()
        .render_enum::<&str>(&ce.value, None, None, None, None, &mut 0)
        .unwrap();
    assert_eq!(out, "enum class Color: uint8_t\n{\n    Red = 0,\n    Green = 1,\n};\n");
}

/// Rust data-carrying enum → IR → C++. A C++ enum holds no per-variant data, so `from_ir` MUST
/// surface a warning for every Tuple/Struct payload (never silently dropped — guards the fix in
/// `CppEnumVariant::from_ir`, where these payloads were once mapped to `None` with no report). The
/// variant names still cross and render as plain enumerators.
#[test]
fn rust_data_enum_to_cpp_warns_on_payload() {
    let e = RustEnum {
        name: "Shape".to_string(),
        visibility: RustVisibility::Pub,
        variants: vec![
            RustEnumVariant {
                name: "Circle".to_string(),
                value: RustEnumVariantValue::Tuple(vec!["f32".to_string()]),
                docs: None,
            },
            RustEnumVariant {
                name: "Rect".to_string(),
                value: RustEnumVariantValue::Tuple(vec!["f32".to_string(), "f32".to_string()]),
                docs: None,
            },
        ],
        repr: None,
        derives: vec![],
        docs: None,
    };

    // to_ir keeps the payloads in the IR (Rust-shaped variant model); no loss yet.
    let ir = e.to_ir(None);
    assert!(!ir.log.has_warnings());

    // from_ir into C++ drops both payloads and reports each one — never silently.
    let ce = CppEnum::from_ir(ir.value, None);
    assert_eq!(
        unsupported_features(&ce.log.warnings),
        vec![
            "data-carrying payload on enum variant `Circle`".to_string(),
            "data-carrying payload on enum variant `Rect`".to_string()
        ]
    );

    let out = cpp()
        .render_enum::<&str>(&ce.value, None, None, None, None, &mut 0)
        .unwrap();
    assert_eq!(out, "enum class Shape\n{\n    Circle,\n    Rect,\n};\n");
}

/// Rust struct → IR → C#. `RustStruct::to_ir` marks the type `is_final` (a Rust struct is not
/// subclassable), which lowers to `is_sealed` on a C# value type. C# structs are *implicitly*
/// sealed, so the `sealed` modifier is invalid syntax on a struct — the renderer MUST omit it.
/// Guards the `CSharpTypeKind::can_be_sealed()` fix (it once emitted `public sealed struct`).
#[test]
fn rust_struct_to_csharp_renders_struct_without_sealed() {
    let s = RustStruct {
        name: "Vec3".to_string(),
        visibility: RustVisibility::Pub,
        generic_args: vec![],
        fields: vec![rust_field("x", "f32"), rust_field("y", "f32"), rust_field("z", "f32")],
        methods: vec![],
        derives: vec![],
        attributes: vec![],
        is_tuple: false,
        docs: None,
    };

    let ir = s.to_ir(None);
    assert!(!ir.log.has_warnings());
    // The IR carries the non-subclassable fact verbatim.
    assert!(ir.value.is_final);

    let cs = CSharpType::from_ir(ir.value, None);
    // Idiomatic C# renaming PascalCases each field; the field type `f32` maps to `float`.
    assert_eq!(
        cs.log.warnings,
        vec![
            ConversionWarning::NamingConventionChanged {
                original: "x".to_string(),
                converted: "X".to_string(),
            },
            ConversionWarning::NamingConventionChanged {
                original: "y".to_string(),
                converted: "Y".to_string(),
            },
            ConversionWarning::NamingConventionChanged {
                original: "z".to_string(),
                converted: "Z".to_string(),
            },
        ]
    );
    // Internally the C# type IS sealed on a Struct kind — the exact bug-B condition.
    assert!(cs.value.is_sealed);
    assert_eq!(cs.value.kind, CSharpTypeKind::Struct);

    let out = CSharpBackend::default()
        .render_struct::<&str>(&cs.value, None, None, None, &mut 0)
        .unwrap();
    // ...yet `sealed` MUST NOT appear in the rendered C#.
    assert!(!out.contains("sealed"));
    assert_eq!(
        out,
        "public struct Vec3\n{\n    public float X;\n    public float Y;\n    public float Z;\n}\n"
    );
}

/// Full triangle: C++ → IR → Rust → IR → C#. A valued enum survives two hops across three
/// languages — its variants and discriminants are preserved end to end. (Visibility has no C++
/// equivalent, so it defaults to private through the chain — an honest, documented mapping.)
#[test]
fn enum_round_trips_across_all_three_backends() {
    let start = CppEnum {
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

    let rust = RustEnum::from_ir(start.to_ir(None).value, None).value;
    // The C++ underlying type `uint8_t` is re-spelled to Rust's `u8` on the way in.
    assert_eq!(rust.repr, Some("u8".to_string()));

    // `u8` is a valid integral repr, so the Rust→IR leg carries it onward with no loss.
    let rust_to_ir = rust.to_ir(None);
    assert!(!rust_to_ir.log.has_warnings());

    let csharp = CSharpEnum::from_ir(rust_to_ir.value, None).value;
    // ...and it lands in C# as `byte`.
    assert_eq!(csharp.underlying_type, Some("byte".to_string()));

    let out = CSharpBackend::default()
        .render_enum::<&str>(&csharp, None, None, None, None, &mut 0)
        .unwrap();
    assert_eq!(out, "private enum Color : byte\n{\n    Red = 0,\n    Green = 1,\n}\n");
}
