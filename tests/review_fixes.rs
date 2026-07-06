//! Exact-value regression tests for the review fixes (B2, B3, B4, S6, S7, N7).

use langprint::backends::BackendItem;
use langprint::backends::cpp_backend::{
    CppBackend, CppBase, CppStruct, CppStructKind, CppVisibility, DocsStyle,
};
use langprint::backends::csharp_backend::{CSharpBackend, CSharpType, CSharpVisibility};
use langprint::backends::rust_backend::{
    RustBackend, RustEnum, RustEnumVariant, RustEnumVariantValue, RustFunction, RustSelfKind,
    RustVisibility,
};
use langprint::conversion::ConversionWarning;
use langprint::ir::{Annotation, LanguageStruct, LanguageStructKind, Visibility};
use langprint::renderers::{FunctionRenderer, StructRenderer};
use langprint::text::{IndentStyle, NewLineStyle};

fn cpp() -> CppBackend {
    CppBackend {
        new_line: NewLineStyle::LF,
        open_brace_on_new_line: true,
        docs_style: DocsStyle::DoubleSlash,
        indent_style: IndentStyle::Spaces,
        indent_size: 4,
        space_before_enum_base: false,
    }
}

fn rust_function(name: &str, self_kind: RustSelfKind, body: Option<Vec<String>>) -> RustFunction {
    RustFunction {
        name: name.to_string(),
        visibility: RustVisibility::Pub,
        self_kind,
        parameters: vec![],
        generic_args: vec![],
        return_type: None,
        is_unsafe: false,
        is_async: false,
        is_const: false,
        abi: None,
        body,
        attributes: vec![],
        docs: None,
        comments: Vec::new(),
    }
}

/// B3: C# `to_ir` must report the lossy collapse of `protected internal` to `Protected`.
#[test]
fn csharp_protected_internal_warns_on_to_ir() {
    let result = CSharpVisibility::ProtectedInternal.to_ir(None);
    assert_eq!(result.value, Visibility::Protected);
    assert_eq!(
        result.log.warnings,
        vec![ConversionWarning::VisibilityApproximated {
            original: "protected internal".to_string(),
            approximated: "Protected".to_string(),
        }]
    );
}

/// B4: a Rust `&mut self` receiver cannot be carried by the IR, so `to_ir` must warn.
#[test]
fn rust_mut_receiver_warns_on_to_ir() {
    let result = rust_function("set", RustSelfKind::RefMut, Some(vec![])).to_ir(None);
    assert_eq!(
        result.log.warnings,
        vec![ConversionWarning::UnsupportedFeature {
            feature: "`&mut self` receiver on method `set`".to_string(),
            resolution: "the IR carries only instance-vs-static; lowered to `&self`".to_string(),
        }]
    );
}

/// S6: only integral `repr`s become an enum underlying type; `repr(C)` is preserved as the curated
/// `Annotation::ReprC` (no longer dropped with a warning).
#[test]
fn rust_repr_c_enum_maps_to_repr_c_annotation() {
    let e = RustEnum {
        name: "E".to_string(),
        visibility: RustVisibility::Pub,
        variants: vec![RustEnumVariant {
            name: "A".to_string(),
            value: RustEnumVariantValue::Unit,
            docs: None,
        }],
        repr: Some("C".to_string()),
        derives: vec![],
        docs: None,
    };
    let result = e.to_ir(None);
    assert_eq!(result.value.underlying_type, None);
    assert!(result.log.warnings.is_empty());
    assert_eq!(result.value.annotations, vec![Annotation::ReprC]);
}

/// N7: a declaration-only Rust function renders as a bare signature, not an `unimplemented!()` body.
#[test]
fn rust_decl_only_function_renders_bare_signature() {
    let out = RustBackend::default()
        .render_function(
            &rust_function("tick", RustSelfKind::Ref, None),
            None::<&str>,
            None::<&str>,
            None,
            &mut 0,
        )
        .unwrap();
    assert_eq!(out, "pub fn tick(&self);\n");
}

/// S7: C++ `Default` base access mirrors the aggregate kind — `private` for a class.
#[test]
fn cpp_class_default_base_is_private() {
    let s = CppStruct {
        struct_kind: CppStructKind::Class,
        is_final: false,
        alignment: None,
        is_packed: false,
        name: "D".to_string(),
        template_params: vec![],
        bases: vec![CppBase {
            name: "B".to_string(),
            visibility: CppVisibility::Default,
        }],
        fields: vec![],
        methods: vec![],
        docs: None,
    };
    let out = cpp()
        .render_struct::<&str>(&s, None, None, None, &mut 0)
        .unwrap();
    assert_eq!(out, "class D : private B\n{\n\n};\n");
}

/// S7: C++ `Default` base access is `public` for a struct.
#[test]
fn cpp_struct_default_base_is_public() {
    let s = CppStruct {
        struct_kind: CppStructKind::Struct,
        is_final: false,
        alignment: None,
        is_packed: false,
        name: "D".to_string(),
        template_params: vec![],
        bases: vec![CppBase {
            name: "B".to_string(),
            visibility: CppVisibility::Default,
        }],
        fields: vec![],
        methods: vec![],
        docs: None,
    };
    let out = cpp()
        .render_struct::<&str>(&s, None, None, None, &mut 0)
        .unwrap();
    assert_eq!(out, "struct D : public B\n{\n\n};\n");
}

/// B2: an abstract IR value type cannot be a C# struct, so `from_ir` lowers it to a class (warned).
#[test]
fn csharp_abstract_struct_lowers_to_class() {
    let ir = LanguageStruct {
        visibility: Visibility::Public,
        struct_kind: LanguageStructKind::Struct,
        is_abstract: true,
        is_final: false,
        name: "Shape".to_string(),
        generic_args: vec![],
        bases: vec![],
        fields: vec![],
        methods: vec![],
        docs: None,
        annotations: Vec::new(),
        raw_attributes: Vec::new(),
    };
    let cs = CSharpType::from_ir(ir, None);
    assert_eq!(
        cs.log.warnings,
        vec![ConversionWarning::UnsupportedFeature {
            feature: "abstract value type `Shape`".to_string(),
            resolution: "C# structs cannot be abstract; lowered to a class".to_string(),
        }]
    );
    let out = CSharpBackend::default()
        .render_struct::<&str>(&cs.value, None, None, None, &mut 0)
        .unwrap();
    assert_eq!(out, "public abstract class Shape\n{\n}\n");
}
