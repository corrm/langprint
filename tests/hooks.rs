//! Opt-in lifecycle hooks (LANGPRINT-9): a custom type override consulted before the `TypeMap`,
//! before/after `to_ir`/`from_ir` mutation hooks on the typed cross-language path, and a render
//! post-process helper. Every hook is off by default; these tests lock the no-op default.

use std::sync::Arc;

use langprint::backends::BackendItem;
use langprint::backends::python_backend::{PythonFunction, PythonFunctionConversionOptions};
use langprint::backends::rust_backend::{
    RustBackend, RustEnum, RustEnumConversionOptions, RustEnumVariant, RustEnumVariantValue,
    RustField, RustFieldConversionOptions, RustFunction, RustFunctionConversionOptions,
    RustParameter, RustSelfKind, RustStruct, RustStructConversionOptions, RustVisibility,
};
use langprint::convert::{ConversionConfig, ConversionHooks};
use langprint::ir::{
    EnumVariant, EnumVariantValue, LanguageEnum, LanguageField, LanguageFunction, LanguageStruct,
    LanguageStructKind, Visibility,
};
use langprint::renderers::{FunctionRenderer, post_process};

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

fn ir_struct(name: &str) -> LanguageStruct {
    LanguageStruct {
        visibility: Visibility::Public,
        struct_kind: LanguageStructKind::Struct,
        is_abstract: false,
        is_final: true,
        name: name.to_string(),
        generic_args: Vec::new(),
        bases: Vec::new(),
        fields: Vec::new(),
        methods: Vec::new(),
        docs: None,
        annotations: Vec::new(),
        raw_attributes: Vec::new(),
    }
}

fn ir_function(name: &str) -> LanguageFunction {
    LanguageFunction {
        name: name.to_string(),
        visibility: Visibility::Public,
        parameters: Vec::new(),
        generic_args: Vec::new(),
        return_type: None,
        is_static: true,
        is_abstract: false,
        is_virtual: false,
        is_override: false,
        is_final: false,
        body: Some(vec!["()".to_string()]),
        docs: None,
        annotations: Vec::new(),
        raw_attributes: Vec::new(),
    }
}

fn native_struct(name: &str) -> RustStruct {
    RustStruct {
        name: name.to_string(),
        visibility: RustVisibility::Pub,
        generic_args: Vec::new(),
        fields: Vec::new(),
        methods: Vec::new(),
        derives: Vec::new(),
        attributes: Vec::new(),
        is_tuple: false,
        docs: None,
    }
}

fn native_function(name: &str) -> RustFunction {
    RustFunction {
        name: name.to_string(),
        visibility: RustVisibility::Pub,
        self_kind: RustSelfKind::None,
        parameters: vec![RustParameter {
            name: "x".to_string(),
            param_type: "i32".to_string(),
        }],
        generic_args: Vec::new(),
        return_type: None,
        is_unsafe: false,
        is_async: false,
        is_const: false,
        abi: None,
        body: Some(vec!["()".to_string()]),
        attributes: Vec::new(),
        docs: None,
    }
}

fn ir_enum(name: &str) -> LanguageEnum {
    LanguageEnum {
        name: name.to_string(),
        visibility: Visibility::Public,
        variants: vec![EnumVariant {
            name: "A".to_string(),
            value: EnumVariantValue::NoValue,
            docs: None,
        }],
        underlying_type: None,
        docs: None,
        annotations: Vec::new(),
        raw_attributes: Vec::new(),
    }
}

fn native_enum(name: &str) -> RustEnum {
    RustEnum {
        name: name.to_string(),
        visibility: RustVisibility::Pub,
        variants: vec![RustEnumVariant {
            name: "A".to_string(),
            value: RustEnumVariantValue::Unit,
            docs: None,
        }],
        repr: None,
        derives: Vec::new(),
        docs: None,
    }
}

#[test]
fn type_override_short_circuits_type_map() {
    // `Handle` is not a TypeMap primitive: without an override it is emitted verbatim with a warning.
    let baseline = RustField::from_ir(ir_field("h", "Handle"), None);
    assert_eq!(baseline.value.field_type, "Handle");
    assert!(baseline.log.has_warnings());

    let config = ConversionConfig {
        type_override: Some(Arc::new(|spelling, _language| {
            (spelling == "Handle").then(|| "IntPtr".to_string())
        })),
        ..Default::default()
    };
    let options = RustFieldConversionOptions { config };

    let overridden = RustField::from_ir(ir_field("h", "Handle"), Some(&options));
    assert_eq!(overridden.value.field_type, "IntPtr");
    assert!(!overridden.log.has_warnings());
}

struct RenameStructHook;
impl ConversionHooks for RenameStructHook {
    fn after_to_ir_struct(&self, s: &mut LanguageStruct) {
        s.name = format!("{}Hooked", s.name);
    }
}

#[test]
fn after_to_ir_struct_hook_fires() {
    // Without hooks the IR name is untouched.
    let plain = native_struct("Widget").to_ir(None);
    assert_eq!(plain.value.name, "Widget");

    let config = ConversionConfig {
        hooks: Some(Arc::new(RenameStructHook)),
        ..Default::default()
    };
    let options = RustStructConversionOptions { config };
    let hooked = native_struct("Widget").to_ir(Some(&options));
    assert_eq!(hooked.value.name, "WidgetHooked");
}

struct RenameFunctionHook;
impl ConversionHooks for RenameFunctionHook {
    fn before_from_ir_function(&self, f: &mut LanguageFunction) {
        f.name = "renamed_before_lowering".to_string();
    }
}

#[test]
fn before_from_ir_function_hook_fires() {
    let config = ConversionConfig {
        hooks: Some(Arc::new(RenameFunctionHook)),
        ..Default::default()
    };
    let options = RustFunctionConversionOptions { config };

    let lowered = RustFunction::from_ir(ir_function("original_name"), Some(&options));
    assert_eq!(lowered.value.name, "renamed_before_lowering");

    let backend = RustBackend::default();
    let mut level = 0;
    let rendered = backend
        .render_function(&lowered.value, None::<&str>, None::<&str>, None, &mut level)
        .unwrap();
    assert!(rendered.contains("renamed_before_lowering"));
}

struct AfterToIrFunctionHook;
impl ConversionHooks for AfterToIrFunctionHook {
    fn after_to_ir_function(&self, f: &mut LanguageFunction) {
        f.name = format!("{}_raised", f.name);
    }
}

#[test]
fn after_to_ir_function_hook_fires() {
    let plain = native_function("compute").to_ir(None);
    assert_eq!(plain.value.name, "compute");

    let config = ConversionConfig {
        hooks: Some(Arc::new(AfterToIrFunctionHook)),
        ..Default::default()
    };
    let options = RustFunctionConversionOptions { config };
    let hooked = native_function("compute").to_ir(Some(&options));
    assert_eq!(hooked.value.name, "compute_raised");
}

struct AfterToIrEnumHook;
impl ConversionHooks for AfterToIrEnumHook {
    fn after_to_ir_enum(&self, e: &mut LanguageEnum) {
        e.name = format!("{}Raised", e.name);
    }
}

#[test]
fn after_to_ir_enum_hook_fires() {
    let plain = native_enum("Color").to_ir(None);
    assert_eq!(plain.value.name, "Color");

    let config = ConversionConfig {
        hooks: Some(Arc::new(AfterToIrEnumHook)),
        ..Default::default()
    };
    let options = RustEnumConversionOptions { config };
    let hooked = native_enum("Color").to_ir(Some(&options));
    assert_eq!(hooked.value.name, "ColorRaised");
}

struct BeforeFromIrStructHook;
impl ConversionHooks for BeforeFromIrStructHook {
    fn before_from_ir_struct(&self, s: &mut LanguageStruct) {
        s.name = "renamed_before_struct_lowering".to_string();
    }
}

#[test]
fn before_from_ir_struct_hook_fires() {
    let plain = RustStruct::from_ir(ir_struct("Player"), None);
    assert_eq!(plain.value.name, "Player");

    let config = ConversionConfig {
        hooks: Some(Arc::new(BeforeFromIrStructHook)),
        ..Default::default()
    };
    let options = RustStructConversionOptions { config };
    let hooked = RustStruct::from_ir(ir_struct("Player"), Some(&options));
    assert_eq!(hooked.value.name, "renamed_before_struct_lowering");
}

struct BeforeFromIrEnumHook;
impl ConversionHooks for BeforeFromIrEnumHook {
    fn before_from_ir_enum(&self, e: &mut LanguageEnum) {
        e.name = "RenamedBeforeEnumLowering".to_string();
    }
}

#[test]
fn before_from_ir_enum_hook_fires() {
    let plain = RustEnum::from_ir(ir_enum("Color"), None);
    assert_eq!(plain.value.name, "Color");

    let config = ConversionConfig {
        hooks: Some(Arc::new(BeforeFromIrEnumHook)),
        ..Default::default()
    };
    let options = RustEnumConversionOptions { config };
    let hooked = RustEnum::from_ir(ir_enum("Color"), Some(&options));
    assert_eq!(hooked.value.name, "RenamedBeforeEnumLowering");
}

#[test]
fn before_from_ir_function_hook_fires_on_python() {
    // Hooks are wired into the untyped Python/Lua/JS path too, not just the typed backends.
    let plain = PythonFunction::from_ir(ir_function("original_name"), None);
    assert_eq!(plain.value.name, "original_name");

    let config = ConversionConfig {
        hooks: Some(Arc::new(RenameFunctionHook)),
        ..Default::default()
    };
    let options = PythonFunctionConversionOptions { config };
    let hooked = PythonFunction::from_ir(ir_function("original_name"), Some(&options));
    assert_eq!(hooked.value.name, "renamed_before_lowering");
}

#[test]
fn render_post_process_wraps_output() {
    let preamble = |rendered: String| format!("#pragma once\n{rendered}");
    let wrapped = post_process("struct Foo {};\n".to_string(), Some(&preamble));
    assert_eq!(wrapped, "#pragma once\nstruct Foo {};\n");

    let untouched = post_process("struct Foo {};\n".to_string(), None);
    assert_eq!(untouched, "struct Foo {};\n");
}

#[test]
fn no_hooks_is_noop() {
    // A default config carries no hooks and no type override; conversion output is byte-identical
    // whether options are absent or present-but-empty.
    let from_none = RustStruct::from_ir(ir_struct("Player"), None).value;
    let options = RustStructConversionOptions {
        config: ConversionConfig::default(),
    };
    let from_default = RustStruct::from_ir(ir_struct("Player"), Some(&options)).value;
    assert_eq!(from_none, from_default);

    // to_ir is likewise unchanged whether options are None or a hook-free Some.
    let to_ir_none = native_enum("Color").to_ir(None).value;
    let enum_options = RustEnumConversionOptions {
        config: ConversionConfig::default(),
    };
    let to_ir_default = native_enum("Color").to_ir(Some(&enum_options)).value;
    assert_eq!(to_ir_none, to_ir_default);

    // The function and enum `from_ir` points are equally inert under a hook-free config.
    let function_options = RustFunctionConversionOptions {
        config: ConversionConfig::default(),
    };
    assert_eq!(
        RustFunction::from_ir(ir_function("calc"), None).value,
        RustFunction::from_ir(ir_function("calc"), Some(&function_options)).value,
    );
    assert_eq!(
        RustEnum::from_ir(ir_enum("Color"), None).value,
        RustEnum::from_ir(ir_enum("Color"), Some(&enum_options)).value,
    );
}
