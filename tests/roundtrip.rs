//! Round-trip property tests: `x == from_ir(to_ir(x))` for the lossless subset.
//!
//! Every `BackendItem` type is tested with a "clean" instance (only common-subset features).
//! If the round-trip is lossless, the value survives `to_ir` → `from_ir` unchanged.

use langprint::backends::BackendItem;
use langprint::backends::cpp_backend::*;
use langprint::backends::csharp_backend::*;
use langprint::backends::rust_backend::*;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn assert_roundtrip<T>(item: T)
where
    T: BackendItem + Clone + PartialEq + std::fmt::Debug,
    T::IrType: std::fmt::Debug,
    T::ConversionOptions: Default,
{
    let ir = item.clone().to_ir(None);
    assert!(
        !ir.log.has_warnings(),
        "to_ir warnings on clean item: {:?}",
        ir.log.warnings
    );
    let back = T::from_ir(ir.value, None);
    assert!(
        !back.log.has_warnings(),
        "from_ir warnings on clean item: {:?}",
        back.log.warnings
    );
    assert_eq!(item, back.value, "round-trip value mismatch");
}

// ── GenericArguments ─────────────────────────────────────────────────────────

#[test]
fn cpp_generic_argument_roundtrips() {
    assert_roundtrip(CppGenericArgument {
        name: "T".into(),
        keyword: "typename".into(),
        default_value: Some("int".into()),
    });
}

#[test]
fn rust_generic_argument_roundtrips() {
    assert_roundtrip(RustGenericArgument {
        name: "T".into(),
        is_lifetime: false,
        const_type: None,
        bounds: Some("Display + Debug".into()),
        default_value: Some("i32".into()),
    });
}

#[test]
fn csharp_generic_argument_roundtrips() {
    assert_roundtrip(CSharpGenericArgument {
        name: "T".into(),
        constraints: Some("class, new()".into()),
    });
}

// ── Visibility ───────────────────────────────────────────────────────────────

#[test]
fn cpp_visibility_roundtrips_public() {
    assert_roundtrip(CppVisibility::Public);
}

#[test]
fn rust_visibility_roundtrips_pub() {
    assert_roundtrip(RustVisibility::Pub);
}

#[test]
fn csharp_visibility_roundtrips_public() {
    assert_roundtrip(CSharpVisibility::Public);
}

// ── Fields ───────────────────────────────────────────────────────────────────

fn clean_cpp_field() -> CppField {
    CppField {
        name: "x".into(),
        field_type: "int32_t".into(),
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

fn clean_rust_field() -> RustField {
    RustField {
        name: "x".into(),
        field_type: "i32".into(),
        visibility: RustVisibility::Pub,
        attributes: Vec::new(),
        docs: None,
    }
}

fn clean_csharp_field() -> CSharpField {
    CSharpField {
        name: "X".into(),
        field_type: "int".into(),
        visibility: CSharpVisibility::Public,
        is_static: false,
        is_const: false,
        is_readonly: false,
        initializer: None,
        attributes: Vec::new(),
        docs: None,
    }
}

#[test]
fn cpp_field_roundtrips() {
    assert_roundtrip(clean_cpp_field());
}

#[test]
fn rust_field_roundtrips() {
    assert_roundtrip(clean_rust_field());
}

#[test]
fn csharp_field_roundtrips() {
    assert_roundtrip(clean_csharp_field());
}

// ── Parameters ───────────────────────────────────────────────────────────────

fn clean_cpp_param() -> CppParameter {
    CppParameter {
        name: "x".into(),
        param_type: "int32_t".into(),
        default_value: None,
    }
}

fn clean_rust_param() -> RustParameter {
    RustParameter {
        name: "x".into(),
        param_type: "i32".into(),
    }
}

fn clean_csharp_param() -> CSharpParameter {
    CSharpParameter {
        name: "x".into(),
        param_type: "int".into(),
        default_value: None,
    }
}

#[test]
fn cpp_parameter_roundtrips() {
    assert_roundtrip(clean_cpp_param());
}

#[test]
fn rust_parameter_roundtrips() {
    assert_roundtrip(clean_rust_param());
}

#[test]
fn csharp_parameter_roundtrips() {
    assert_roundtrip(clean_csharp_param());
}

// ── Functions ────────────────────────────────────────────────────────────────

fn clean_cpp_function() -> CppFunction {
    CppFunction {
        name: "foo".into(),
        parent_name: None,
        visibility: CppVisibility::Public,
        parameters: vec![clean_cpp_param()],
        template_params: vec![],
        return_type: Some("void".into()),
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
        body: None,
        docs: None,
    }
}

fn clean_rust_function() -> RustFunction {
    RustFunction {
        name: "foo".into(),
        visibility: RustVisibility::Pub,
        self_kind: RustSelfKind::None,
        parameters: vec![clean_rust_param()],
        generic_args: vec![],
        return_type: None,
        is_unsafe: false,
        is_async: false,
        is_const: false,
        body: None,
        attributes: Vec::new(),
        docs: None,
    }
}

fn clean_csharp_method() -> CSharpMethod {
    CSharpMethod {
        name: "Foo".into(),
        visibility: CSharpVisibility::Public,
        parameters: vec![clean_csharp_param()],
        generic_args: vec![],
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

#[test]
fn cpp_function_roundtrips() {
    assert_roundtrip(clean_cpp_function());
}

#[test]
fn rust_function_roundtrips() {
    assert_roundtrip(clean_rust_function());
}

#[test]
fn csharp_method_roundtrips() {
    assert_roundtrip(clean_csharp_method());
}

// ── Enums ────────────────────────────────────────────────────────────────────

fn clean_cpp_enum_variant() -> CppEnumVariant {
    CppEnumVariant {
        name: "North".into(),
        value: Some("0".to_string()),
        docs: None,
    }
}

fn clean_cpp_enum() -> CppEnum {
    CppEnum {
        name: "Direction".into(),
        variants: vec![clean_cpp_enum_variant()],
        is_enum_class: true,
        underlying_type: None,
        docs: None,
    }
}

fn clean_rust_enum() -> RustEnum {
    RustEnum {
        name: "Direction".into(),
        visibility: RustVisibility::Pub,
        variants: vec![RustEnumVariant {
            name: "North".into(),
            value: RustEnumVariantValue::Unit,
            docs: None,
        }],
        repr: None,
        derives: Vec::new(),
        docs: None,
    }
}

fn clean_csharp_enum() -> CSharpEnum {
    CSharpEnum {
        name: "Direction".into(),
        visibility: CSharpVisibility::Public,
        underlying_type: None,
        members: vec![CSharpEnumMember {
            name: "North".into(),
            value: Some("0".to_string()),
            docs: None,
        }],
        is_flags: false,
        attributes: Vec::new(),
        docs: None,
    }
}

#[test]
fn cpp_enum_variant_roundtrips() {
    assert_roundtrip(clean_cpp_enum_variant());
}

#[test]
fn cpp_enum_roundtrips() {
    assert_roundtrip(clean_cpp_enum());
}

#[test]
fn rust_enum_roundtrips() {
    assert_roundtrip(clean_rust_enum());
}

#[test]
fn csharp_enum_roundtrips() {
    assert_roundtrip(clean_csharp_enum());
}

// ── Structs ──────────────────────────────────────────────────────────────────

fn clean_cpp_struct() -> CppStruct {
    CppStruct {
        struct_kind: CppStructKind::Struct,
        is_final: false,
        alignment: None,
        name: "Point".into(),
        template_params: vec![],
        bases: vec![],
        fields: vec![clean_cpp_field()],
        methods: vec![],
        docs: None,
    }
}

fn clean_rust_struct() -> RustStruct {
    RustStruct {
        name: "Point".into(),
        visibility: RustVisibility::Pub,
        generic_args: vec![],
        fields: vec![clean_rust_field()],
        methods: vec![],
        derives: Vec::new(),
        attributes: Vec::new(),
        is_tuple: false,
        docs: None,
    }
}

fn clean_csharp_type() -> CSharpType {
    CSharpType {
        kind: CSharpTypeKind::Struct,
        name: "Point".into(),
        visibility: CSharpVisibility::Public,
        is_abstract: false,
        is_sealed: false,
        is_static: false,
        is_partial: false,
        generic_args: vec![],
        base_class: None,
        interfaces: vec![],
        fields: vec![clean_csharp_field()],
        properties: vec![],
        methods: vec![],
        attributes: Vec::new(),
        docs: None,
    }
}

#[test]
fn cpp_struct_roundtrips() {
    assert_roundtrip(clean_cpp_struct());
}

#[test]
fn rust_struct_roundtrips() {
    assert_roundtrip(clean_rust_struct());
}

#[test]
fn csharp_type_roundtrips() {
    assert_roundtrip(clean_csharp_type());
}

// ── Constants ────────────────────────────────────────────────────────────────

fn clean_cpp_constant() -> CppConstant {
    CppConstant {
        name: "MAX".into(),
        visibility: CppVisibility::Public,
        data_type: "int".into(),
        value: "10".into(),
        docs: None,
    }
}

fn clean_rust_constant() -> RustConstant {
    RustConstant {
        name: "MAX".into(),
        visibility: RustVisibility::Pub,
        data_type: "i32".into(),
        value: "10".into(),
        is_static: false,
        docs: None,
    }
}

fn clean_csharp_constant() -> CSharpConstant {
    CSharpConstant {
        name: "MAX".into(),
        visibility: CSharpVisibility::Public,
        data_type: "int".into(),
        value: "10".into(),
        docs: None,
    }
}

#[test]
fn cpp_constant_roundtrips() {
    assert_roundtrip(clean_cpp_constant());
}

#[test]
fn rust_constant_roundtrips() {
    assert_roundtrip(clean_rust_constant());
}

#[test]
fn csharp_constant_roundtrips() {
    assert_roundtrip(clean_csharp_constant());
}

// ── Defines ──────────────────────────────────────────────────────────────────

fn clean_cpp_define() -> CppDefinition {
    CppDefinition {
        name: "DEBUG".into(),
        value: Some("1".into()),
        docs: None,
    }
}

fn clean_rust_define() -> RustDefinition {
    RustDefinition {
        name: "DEBUG".into(),
        value: Some("1".into()),
        docs: None,
    }
}

fn clean_csharp_define() -> CSharpDefinition {
    CSharpDefinition {
        name: "DEBUG".into(),
        value: Some("1".into()),
        docs: None,
    }
}

#[test]
fn cpp_define_roundtrips() {
    assert_roundtrip(clean_cpp_define());
}

#[test]
fn rust_define_roundtrips() {
    assert_roundtrip(clean_rust_define());
}

#[test]
fn csharp_define_roundtrips() {
    assert_roundtrip(clean_csharp_define());
}

// ── Namespaces ───────────────────────────────────────────────────────────────

fn clean_cpp_namespace() -> CppNamespace {
    CppNamespace {
        name: "mylib".into(),
        defines: None,
        constants: None,
        enums: None,
        structs: None,
        functions: None,
        namespaces: None,
    }
}

fn clean_rust_module() -> RustModule {
    RustModule {
        name: "mylib".into(),
        visibility: RustVisibility::Pub,
        defines: None,
        constants: None,
        enums: None,
        structs: None,
        functions: None,
        modules: None,
        docs: None,
    }
}

fn clean_csharp_namespace() -> CSharpNamespace {
    CSharpNamespace {
        name: "MyLib".into(),
        defines: None,
        constants: None,
        enums: None,
        types: None,
        namespaces: None,
        file_scoped: false,
        docs: None,
    }
}

#[test]
fn cpp_namespace_roundtrips() {
    assert_roundtrip(clean_cpp_namespace());
}

#[test]
fn rust_module_roundtrips() {
    assert_roundtrip(clean_rust_module());
}

#[test]
fn csharp_namespace_roundtrips() {
    assert_roundtrip(clean_csharp_namespace());
}
