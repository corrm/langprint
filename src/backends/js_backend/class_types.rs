use crate::{
    backends::BackendItem,
    conversion::{dropped_annotations_warning, ConversionLog, ConversionResult, ConversionWarning},
    convert::{rename_identifier, ConversionConfig, IdentifierKind},
    ir::{LanguageBase, LanguageField, LanguageStruct, LanguageStructKind, Visibility},
    type_map::TargetLanguage,
};

use super::{JsFunction, JsFunctionConversionOptions};

/// A class field (`name = value;` or `static name = value;`).
///
/// JavaScript class fields are plain assignments with a free-form right-hand
/// side, so both name and value are modelled as raw strings.
#[derive(Debug, Clone, PartialEq)]
pub struct JsField {
    /// The name of the field.
    pub name: String,
    /// The assigned value, rendered verbatim (e.g. `0`, `"default"`, `null`).
    pub value: String,
    /// `true` to render the field with a leading `static` keyword.
    pub is_static: bool,
}

/// Represents a JavaScript class (`class Name {` / `class Name extends Base {`).
#[derive(Debug, Clone, PartialEq)]
pub struct JsClass {
    /// The name of the class.
    pub name: String,
    /// The base class to extend, rendered verbatim after `extends`.
    pub extends: Option<String>,
    /// Class fields.
    pub fields: Vec<JsField>,
    /// The methods of the class.
    pub methods: Vec<JsFunction>,
    /// Optional free-form JSDoc description text, rendered as the first line of the block.
    pub doc: Option<String>,
}

impl BackendItem for JsClass {
    type IrType = LanguageStruct;
    type ConversionOptions = JsClassConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut log = ConversionLog::new();

        let fields = self
            .fields
            .into_iter()
            .map(|field| LanguageField {
                name: field.name,
                field_type: String::new(),
                visibility: Visibility::Public,
                is_static: field.is_static,
                is_const: false,
                docs: None,
                annotations: Vec::new(),
                raw_attributes: Vec::new(),
            })
            .collect();

        let mut methods = Vec::with_capacity(self.methods.len());
        for method in self.methods {
            let result = method.to_ir(None);
            log.add_warnings(result.log.warnings);
            methods.push(result.value);
        }

        let bases = self
            .extends
            .into_iter()
            .map(|name| LanguageBase {
                name,
                visibility: Visibility::Public,
            })
            .collect();

        ConversionResult::with_log(
            LanguageStruct {
                visibility: Visibility::Public,
                struct_kind: LanguageStructKind::Class,
                is_abstract: false,
                is_final: false,
                name: self.name,
                generic_args: Vec::new(),
                bases,
                fields,
                methods,
                docs: self.doc.map(|doc| vec![doc]),
                annotations: Vec::new(),
                raw_attributes: Vec::new(),
            },
            log,
        )
    }

    fn from_ir(input: Self::IrType, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();
        let config = options.map(|options| options.config.clone()).unwrap_or_default();

        let name = rename_identifier(&config, &input.name, TargetLanguage::Js, IdentifierKind::Type);
        log.add_warnings(name.log.warnings);

        if !input.annotations.is_empty() || !input.raw_attributes.is_empty() {
            log.add_warning(dropped_annotations_warning(
                input.annotations.len() + input.raw_attributes.len(),
                "class",
                &input.name,
                "JavaScript",
            ));
        }

        let mut fields = Vec::with_capacity(input.fields.len());
        for field in input.fields {
            let field_name = rename_identifier(&config, &field.name, TargetLanguage::Js, IdentifierKind::Field);
            log.add_warnings(field_name.log.warnings);
            fields.push(JsField {
                name: field_name.value,
                value: "null".to_string(),
                is_static: field.is_static,
            });
        }

        let function_options = JsFunctionConversionOptions { config: config.clone() };
        let mut methods = Vec::with_capacity(input.methods.len());
        for method in input.methods {
            let result = JsFunction::from_ir(method, Some(&function_options));
            log.add_warnings(result.log.warnings);
            methods.push(result.value);
        }

        if input.bases.len() > 1 {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("{} base classes on `{}`", input.bases.len(), input.name),
                resolution: "JavaScript has single inheritance; kept the first base, dropped the rest".to_string(),
            });
        }
        let extends = input.bases.into_iter().next().map(|base| base.name);

        ConversionResult::with_log(
            JsClass {
                name: name.value,
                extends,
                fields,
                methods,
                doc: input.docs.map(|docs| docs.join("\n")),
            },
            log,
        )
    }
}

/// Conversion options for JavaScript classes.
#[derive(Debug, Clone, Default)]
pub struct JsClassConversionOptions {
    /// Cross-language conversion configuration.
    pub config: ConversionConfig,
}

/// Render options for JavaScript classes.
#[derive(Debug, Clone)]
pub struct JsClassRenderOptions {
    /// Whether to render a JSDoc block for the class and its methods when type
    /// information is present.
    pub render_jsdoc: bool,
}

impl Default for JsClassRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl JsClassRenderOptions {
    pub const DEFAULT: Self = Self { render_jsdoc: true };
}
