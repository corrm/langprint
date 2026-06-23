//! Render and conversion tests for the C# backend.

use langprint::backends::BackendItem;
use langprint::backends::csharp_backend::{
    CSharpBackend, CSharpConstant, CSharpEnum, CSharpEnumMember, CSharpField, CSharpMethod, CSharpParameter,
    CSharpProperty, CSharpType, CSharpTypeKind, CSharpVisibility,
};
use langprint::ir::{EnumVariant, EnumVariantValue, LanguageEnum, Visibility};
use langprint::renderers::{ConstantRenderer, EnumRenderer, FunctionRenderer, StructRenderer};

fn backend() -> CSharpBackend {
    CSharpBackend::default()
}

fn field(name: &str, ty: &str) -> CSharpField {
    CSharpField {
        name: name.to_string(),
        field_type: ty.to_string(),
        visibility: CSharpVisibility::Public,
        is_static: false,
        is_const: false,
        is_readonly: false,
        initializer: None,
        attributes: Vec::new(),
        docs: None,
    }
}

fn method(name: &str) -> CSharpMethod {
    CSharpMethod {
        name: name.to_string(),
        visibility: CSharpVisibility::Public,
        parameters: Vec::new(),
        generic_args: Vec::new(),
        return_type: None,
        is_static: false,
        is_abstract: false,
        is_virtual: false,
        is_override: false,
        is_sealed: false,
        is_async: false,
        body: None,
        attributes: Vec::new(),
        docs: None,
    }
}

fn empty_type(kind: CSharpTypeKind, name: &str) -> CSharpType {
    CSharpType {
        kind,
        name: name.to_string(),
        visibility: CSharpVisibility::Public,
        is_abstract: false,
        is_sealed: false,
        is_static: false,
        is_partial: false,
        generic_args: Vec::new(),
        base_class: None,
        interfaces: Vec::new(),
        fields: Vec::new(),
        properties: Vec::new(),
        methods: Vec::new(),
        attributes: Vec::new(),
        docs: None,
    }
}

#[test]
fn renders_class_with_base_interface_field_and_method() {
    let mut heal = method("Heal");
    heal.parameters.push(CSharpParameter {
        name: "amount".to_string(),
        param_type: "float".to_string(),
        default_value: None,
    });
    heal.body = Some(vec!["this.health += amount;".to_string()]);

    let mut ty = empty_type(CSharpTypeKind::Class, "Player");
    ty.base_class = Some("Entity".to_string());
    ty.interfaces.push("IDamageable".to_string());
    ty.fields.push(field("health", "float"));
    ty.methods.push(heal);

    let rendered = backend().render_struct::<&str>(&ty, None, None, None, &mut 0).unwrap();
    assert_eq!(
        rendered,
        "public class Player : Entity, IDamageable\n{\n    public float health;\n\n    public void Heal(float amount)\n    {\n        this.health += amount;\n    }\n}\n"
    );
}

#[test]
fn renders_struct_with_fields() {
    let mut ty = empty_type(CSharpTypeKind::Struct, "Vec2");
    ty.fields.push(field("x", "float"));
    ty.fields.push(field("y", "float"));

    let rendered = backend().render_struct::<&str>(&ty, None, None, None, &mut 0).unwrap();
    assert_eq!(
        rendered,
        "public struct Vec2\n{\n    public float x;\n    public float y;\n}\n"
    );
}

#[test]
fn renders_interface_with_abstract_method() {
    let mut run = method("Run");
    run.visibility = CSharpVisibility::Default;
    let mut ty = empty_type(CSharpTypeKind::Interface, "IService");
    ty.methods.push(run);

    let rendered = backend().render_struct::<&str>(&ty, None, None, None, &mut 0).unwrap();
    assert_eq!(rendered, "public interface IService\n{\n    void Run();\n}\n");
}

#[test]
fn renders_auto_property() {
    let mut ty = empty_type(CSharpTypeKind::Class, "Bag");
    ty.properties.push(CSharpProperty {
        name: "Count".to_string(),
        prop_type: "int".to_string(),
        visibility: CSharpVisibility::Public,
        is_static: false,
        has_getter: true,
        has_setter: true,
        getter_body: None,
        setter_body: None,
        docs: None,
    });

    let rendered = backend().render_struct::<&str>(&ty, None, None, None, &mut 0).unwrap();
    assert_eq!(rendered, "public class Bag\n{\n    public int Count { get; set; }\n}\n");
}

#[test]
fn renders_flags_enum() {
    let enum_ = CSharpEnum {
        name: "Access".to_string(),
        visibility: CSharpVisibility::Public,
        underlying_type: Some("byte".to_string()),
        members: vec![
            CSharpEnumMember {
                name: "None".to_string(),
                value: Some("0".to_string()),
                docs: None,
            },
            CSharpEnumMember {
                name: "Read".to_string(),
                value: Some("1".to_string()),
                docs: None,
            },
            CSharpEnumMember {
                name: "Write".to_string(),
                value: Some("2".to_string()),
                docs: None,
            },
        ],
        is_flags: true,
        attributes: Vec::new(),
        docs: None,
    };

    let rendered = backend()
        .render_enum::<&str>(&enum_, None, None, None, None, &mut 0)
        .unwrap();
    assert_eq!(
        rendered,
        "[Flags]\npublic enum Access : byte\n{\n    None = 0,\n    Read = 1,\n    Write = 2,\n}\n"
    );
}

#[test]
fn renders_constant() {
    let constant = CSharpConstant {
        name: "MaxHealth".to_string(),
        visibility: CSharpVisibility::Public,
        data_type: "int".to_string(),
        value: "100".to_string(),
        docs: None,
    };
    let rendered = backend()
        .render_constant::<&str>(&constant, None, None, None, &mut 0)
        .unwrap();
    assert_eq!(rendered, "public const int MaxHealth = 100;\n");
}

#[test]
fn renders_static_method() {
    let mut add = method("Add");
    add.is_static = true;
    add.return_type = Some("int".to_string());
    add.parameters = vec![
        CSharpParameter {
            name: "a".to_string(),
            param_type: "int".to_string(),
            default_value: None,
        },
        CSharpParameter {
            name: "b".to_string(),
            param_type: "int".to_string(),
            default_value: None,
        },
    ];
    add.body = Some(vec!["return a + b;".to_string()]);

    let rendered = backend()
        .render_function::<&str>(&add, None, None, None, &mut 0)
        .unwrap();
    assert_eq!(
        rendered,
        "public static int Add(int a, int b)\n{\n    return a + b;\n}\n"
    );
}

#[test]
fn class_round_trips_through_ir() {
    let mut heal = method("Heal");
    heal.body = Some(vec!["// body".to_string()]);
    let mut ty = empty_type(CSharpTypeKind::Class, "Player");
    ty.base_class = Some("Entity".to_string());
    ty.interfaces.push("IDamageable".to_string());
    ty.fields.push(field("health", "float"));
    ty.methods.push(heal);

    let ir = ty.to_ir(None).value;
    let back = CSharpType::from_ir(ir, None).value;

    assert_eq!(back.kind, CSharpTypeKind::Class);
    assert_eq!(back.name, "Player");
    assert_eq!(back.base_class.as_deref(), Some("Entity"));
    assert_eq!(back.interfaces, vec!["IDamageable".to_string()]);
    assert_eq!(back.fields.len(), 1);
    assert_eq!(back.fields[0].name, "health");
    assert_eq!(back.fields[0].field_type, "float");
    assert_eq!(back.methods.len(), 1);
    assert_eq!(back.methods[0].name, "Heal");
    assert_eq!(
        back.methods[0].body.as_deref(),
        Some(["// body".to_string()].as_slice())
    );
}

#[test]
fn to_ir_warns_on_property_readonly_async_and_flags() {
    // Property on a type → lowered to a field, with a warning.
    let mut ty = empty_type(CSharpTypeKind::Class, "Bag");
    ty.properties.push(CSharpProperty {
        name: "Count".to_string(),
        prop_type: "int".to_string(),
        visibility: CSharpVisibility::Public,
        is_static: false,
        has_getter: true,
        has_setter: true,
        getter_body: None,
        setter_body: None,
        docs: None,
    });
    let result = ty.to_ir(None);
    assert_eq!(result.log.warnings.len(), 1);
    assert_eq!(result.value.fields.len(), 1);
    assert_eq!(result.value.fields[0].name, "Count");

    // readonly field → warning.
    let mut ro = field("id", "int");
    ro.is_readonly = true;
    assert_eq!(ro.to_ir(None).log.warnings.len(), 1);

    // async method → warning.
    let mut run = method("Run");
    run.is_async = true;
    assert_eq!(run.to_ir(None).log.warnings.len(), 1);

    // [Flags] enum → warning.
    let flags_enum = CSharpEnum {
        name: "E".to_string(),
        visibility: CSharpVisibility::Public,
        underlying_type: None,
        members: Vec::new(),
        is_flags: true,
        attributes: Vec::new(),
        docs: None,
    };
    assert_eq!(flags_enum.to_ir(None).log.warnings.len(), 1);
}

#[test]
fn from_ir_warns_on_data_carrying_enum_variant() {
    let language_enum = LanguageEnum {
        name: "Shape".to_string(),
        visibility: Visibility::Public,
        variants: vec![
            EnumVariant {
                name: "None".to_string(),
                value: EnumVariantValue::NoValue,
                docs: None,
            },
            EnumVariant {
                name: "Circle".to_string(),
                value: EnumVariantValue::Tuple(vec!["float".to_string()]),
                docs: None,
            },
        ],
        underlying_type: None,
        docs: None,
    };
    let result = CSharpEnum::from_ir(language_enum, None);
    assert_eq!(result.log.warnings.len(), 1);
    assert_eq!(result.value.members.len(), 2);
    assert_eq!(result.value.members[1].name, "Circle");
    assert_eq!(result.value.members[1].value, None);
}
