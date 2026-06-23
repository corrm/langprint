//! Demonstrates langprint's two layers: rich per-language native models that build and render
//! directly, and the neutral declaration IR used as a lossy bridge for cross-language conversion.
//!
//! Run with: `cargo run -p langprint --example cross_language`

use langprint::backends::BackendItem;
use langprint::backends::csharp_backend::{
    CSharpBackend, CSharpField, CSharpType, CSharpTypeConversionOptions, CSharpTypeKind, CSharpVisibility,
};
use langprint::backends::rust_backend::{RustBackend, RustField, RustStruct, RustVisibility};
use langprint::conversion::ConversionWarning;
use langprint::renderers::StructRenderer;
use langprint::{ConversionConfig, PrimitiveType, TargetLanguage, TypeMap};

fn rust_field(name: &str, ty: &str) -> RustField {
    RustField {
        name: name.to_string(),
        field_type: ty.to_string(),
        visibility: RustVisibility::Pub,
        attributes: vec![],
        docs: None,
    }
}

fn print_warnings(label: &str, warnings: &[ConversionWarning]) {
    if warnings.is_empty() {
        println!("({label}: no features lost)");
        return;
    }
    println!("({label}: lossy — each dropped feature is reported, never silent)");
    for warning in warnings {
        if let ConversionWarning::UnsupportedFeature { feature, resolution } = warning {
            println!("  - {feature}: {resolution}");
        }
    }
}

fn main() {
    // Layer 1: build a native Rust model and render it directly. No IR involved.
    let player = RustStruct {
        name: "Player".to_string(),
        visibility: RustVisibility::Pub,
        generic_args: vec![],
        fields: vec![rust_field("health", "f32"), rust_field("mana", "f32")],
        methods: vec![],
        derives: vec![],
        attributes: vec![],
        is_tuple: false,
        docs: None,
    };

    let rust_src = RustBackend::default()
        .render_struct(&player, None::<&str>, None::<&str>, None, &mut 0)
        .expect("render Rust");
    println!("== Rust (native) ==\n{rust_src}");

    // Layer 2: cross-language. Lower the native model into the neutral IR, then raise it into a
    // different backend's native model. Each leg reports the features it cannot carry.
    let to_ir = player.to_ir(None);
    print_warnings("Rust -> IR", &to_ir.log.warnings);

    let csharp = CSharpType::from_ir(to_ir.value, None);
    print_warnings("IR -> C#", &csharp.log.warnings);

    let csharp_src = CSharpBackend::default()
        .render_struct::<&str>(&csharp.value, None, None, None, &mut 0)
        .expect("render C#");
    println!("\n== C# (converted from the Rust declaration) ==\n{csharp_src}");

    // A lossy direction: a C# class with a base and an interface has no Rust equivalent. Both are
    // reported as dropped (Rust has no inheritance), while the fields cross cleanly.
    let entity = CSharpType {
        kind: CSharpTypeKind::Class,
        name: "Boss".to_string(),
        visibility: CSharpVisibility::Public,
        is_abstract: false,
        is_sealed: false,
        is_static: false,
        is_partial: false,
        generic_args: vec![],
        base_class: Some("Entity".to_string()),
        interfaces: vec!["IDamageable".to_string()],
        fields: vec![CSharpField {
            name: "health".to_string(),
            field_type: "float".to_string(),
            visibility: CSharpVisibility::Public,
            is_static: false,
            is_const: false,
            is_readonly: false,
            initializer: None,
            attributes: vec![],
            docs: None,
        }],
        properties: vec![],
        methods: vec![],
        attributes: vec![],
        docs: None,
    };

    let entity_ir = entity.to_ir(None);
    print_warnings("C# -> IR", &entity_ir.log.warnings);

    let rust_boss = RustStruct::from_ir(entity_ir.value, None);
    print_warnings("IR -> Rust", &rust_boss.log.warnings);

    let boss_src = RustBackend::default()
        .render_struct(&rust_boss.value, None::<&str>, None::<&str>, None, &mut 0)
        .expect("render Rust");
    println!("\n== Rust (converted from the C# class — inheritance dropped) ==\n{boss_src}");

    // Customize the conversion: extend the built-in TypeMap with a game type, override an output
    // spelling, and turn off idiomatic renaming. The map is exposed for exactly this.
    let mut type_map = TypeMap::builtin();
    type_map.insert_spelling("FString", PrimitiveType::Str);
    type_map.set_output(PrimitiveType::Str, TargetLanguage::CSharp, "string");
    let config = ConversionConfig::new(type_map, false);
    let options = CSharpTypeConversionOptions { config };

    let actor = RustStruct {
        name: "Actor".to_string(),
        visibility: RustVisibility::Pub,
        generic_args: vec![],
        fields: vec![rust_field("display_name", "FString"), rust_field("health", "f32")],
        methods: vec![],
        derives: vec![],
        attributes: vec![],
        is_tuple: false,
        docs: None,
    };
    let actor_cs = CSharpType::from_ir(actor.to_ir(None).value, Some(&options));
    let actor_src = CSharpBackend::default()
        .render_struct::<&str>(&actor_cs.value, None, None, None, &mut 0)
        .expect("render C#");
    println!("\n== C# (custom TypeMap: FString->string; renaming off) ==\n{actor_src}");
}
