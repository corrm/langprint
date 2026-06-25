use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    convert::ConversionConfig,
    ir::{LanguageStruct, LanguageStructKind, RawAttribute},
    type_map::TargetLanguage,
};

use super::attributes::rust_attribute_to_annotation;

use super::{
    RustField, RustFieldConversionOptions, RustFunction, RustFunctionConversionOptions, RustGenericArgument,
    RustVisibility,
};

/// Represents a Rust struct, together with the methods rendered in its `impl` block.
#[derive(Debug, Clone, PartialEq)]
pub struct RustStruct {
    /// The name of the struct.
    pub name: String,
    /// The visibility of the struct.
    pub visibility: RustVisibility,
    /// Generic parameters of the struct.
    pub generic_args: Vec<RustGenericArgument>,
    /// The fields of the struct.
    pub fields: Vec<RustField>,
    /// The methods rendered in an inherent `impl` block.
    pub methods: Vec<RustFunction>,
    /// Derives applied to the struct (e.g. `Debug`, `Clone`).
    pub derives: Vec<String>,
    /// Attributes applied to the struct (without the leading `#[`, e.g. `repr(C)`).
    pub attributes: Vec<String>,
    /// Whether the struct is a tuple struct (`struct Foo(A, B);`).
    pub is_tuple: bool,
    /// Optional documentation for the struct.
    pub docs: Option<Vec<String>>,
}

impl BackendItem for RustStruct {
    type IrType = LanguageStruct;
    type ConversionOptions = RustStructConversionOptions;

    fn to_ir(self, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut log = ConversionLog::new();

        let mut annotations = Vec::new();
        let mut raw_attributes = Vec::new();
        for derive in &self.derives {
            raw_attributes.push(RawAttribute {
                source: TargetLanguage::Rust,
                text: format!("derive({derive})"),
            });
        }
        for attribute in &self.attributes {
            match rust_attribute_to_annotation(attribute) {
                Some(annotation) => annotations.push(annotation),
                None => raw_attributes.push(RawAttribute {
                    source: TargetLanguage::Rust,
                    text: attribute.clone(),
                }),
            }
        }
        if self.is_tuple {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("tuple struct `{}`", self.name),
                resolution: "lowered to a named-field struct in the language-agnostic IR".to_string(),
            });
        }

        let visibility = self.visibility.to_ir(None);
        log.add_warnings(visibility.log.warnings);

        let mut fields = Vec::with_capacity(self.fields.len());
        for field in self.fields {
            let result = field.to_ir(None);
            log.add_warnings(result.log.warnings);
            fields.push(result.value);
        }

        let mut methods = Vec::with_capacity(self.methods.len());
        for method in self.methods {
            let result = method.to_ir(None);
            log.add_warnings(result.log.warnings);
            methods.push(result.value);
        }

        let mut generic_args = Vec::with_capacity(self.generic_args.len());
        for generic in self.generic_args {
            let result = generic.to_ir(None);
            log.add_warnings(result.log.warnings);
            generic_args.push(result.value);
        }

        let mut ir = LanguageStruct {
            visibility: visibility.value,
            struct_kind: LanguageStructKind::Struct,
            is_abstract: false,
            // A Rust struct cannot be subclassed, so it is final in the IR's inheritance model.
            is_final: true,
            name: self.name,
            generic_args,
            bases: Vec::new(),
            fields,
            methods,
            docs: self.docs,
            annotations,
            raw_attributes,
        };
        if let Some(hooks) = options.and_then(|options| options.config.hooks.as_ref()) {
            hooks.after_to_ir_struct(&mut ir);
        }

        ConversionResult::with_log(ir, log)
    }

    fn from_ir(mut input: Self::IrType, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();
        let config = options.map(|options| options.config.clone()).unwrap_or_default();
        if let Some(hooks) = &config.hooks {
            hooks.before_from_ir_struct(&mut input);
        }

        if input.struct_kind == LanguageStructKind::Union {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("union `{}`", input.name),
                resolution: "lowered to a Rust struct (use a `union` manually for C layout)".to_string(),
            });
        }
        if input.is_abstract {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("abstract type `{}`", input.name),
                resolution: "Rust structs cannot be abstract; consider a trait".to_string(),
            });
        }
        for base in &input.bases {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("base `{}` of `{}`", base.name, input.name),
                resolution: "Rust has no inheritance; the base was dropped (use composition/traits)".to_string(),
            });
        }

        let visibility = RustVisibility::from_ir(input.visibility, None);
        log.add_warnings(visibility.log.warnings);

        let mut derives = Vec::new();
        let mut attributes = Vec::new();
        for annotation in &input.annotations {
            if let Some(rendered) = config.annotation_map.resolve(TargetLanguage::Rust, annotation) {
                attributes.push(rendered);
            }
        }
        for raw in &input.raw_attributes {
            if raw.source != TargetLanguage::Rust {
                log.add_warning(ConversionWarning::UnsupportedFeature {
                    feature: format!("opaque {:?} attribute `{}`", raw.source, raw.text),
                    resolution: "cannot translate to Rust; dropped".to_string(),
                });
                continue;
            }
            match raw.text.strip_prefix("derive(").and_then(|rest| rest.strip_suffix(")")) {
                Some(derive) => derives.push(derive.to_string()),
                None => attributes.push(raw.text.clone()),
            }
        }

        let field_options = RustFieldConversionOptions { config: config.clone() };
        let mut fields = Vec::with_capacity(input.fields.len());
        for field in input.fields {
            let result = RustField::from_ir(field, Some(&field_options));
            log.add_warnings(result.log.warnings);
            fields.push(result.value);
        }

        let function_options = RustFunctionConversionOptions { config: config.clone() };
        let mut methods = Vec::with_capacity(input.methods.len());
        for method in input.methods {
            let result = RustFunction::from_ir(method, Some(&function_options));
            log.add_warnings(result.log.warnings);
            methods.push(result.value);
        }

        let mut generic_args = Vec::with_capacity(input.generic_args.len());
        for generic in &input.generic_args {
            let result = RustGenericArgument::from_ir(generic.clone(), None);
            log.add_warnings(result.log.warnings);
            generic_args.push(result.value);
        }

        ConversionResult::with_log(
            RustStruct {
                name: input.name,
                visibility: visibility.value,
                generic_args,
                fields,
                methods,
                derives,
                attributes,
                is_tuple: false,
                docs: input.docs,
            },
            log,
        )
    }
}

/// Conversion options for Rust structs.
#[derive(Debug, Clone, Default)]
pub struct RustStructConversionOptions {
    /// Cross-language conversion configuration (type mapping + renaming).
    pub config: ConversionConfig,
}

/// Render options for Rust structs.
#[derive(Debug, Clone)]
pub struct RustStructRenderOptions {
    /// Whether to render documentation comments.
    pub render_docs: bool,
    /// Whether to render derives and attributes.
    pub render_attributes: bool,
    /// Whether to render an `impl` block for the methods.
    pub render_impl: bool,
}

impl Default for RustStructRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl RustStructRenderOptions {
    pub const DEFAULT: Self = Self {
        render_docs: true,
        render_attributes: true,
        render_impl: true,
    };
}
