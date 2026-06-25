use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult},
    convert::ConversionConfig,
    ir::{LanguageFunction, LanguageFunctionParameter, Visibility},
};

/// Represents a parameter of a JavaScript function.
///
/// JavaScript has no type annotations in the surface syntax, so the signature
/// carries only the name and an optional default value. `type_doc` is a free-form
/// type string that exists solely to feed an optional JSDoc `@param {type}` tag; it
/// is never emitted in the signature itself.
#[derive(Debug, Clone, PartialEq)]
pub struct JsParameter {
    /// The name of the parameter.
    pub name: String,
    /// The default value of the parameter, if any (e.g. `0`), rendered as `name = value`.
    pub default: Option<String>,
    /// The type string for JSDoc only (e.g. `number`); never emitted in the signature.
    pub type_doc: Option<String>,
}

impl BackendItem for JsParameter {
    type IrType = LanguageFunctionParameter;
    type ConversionOptions = JsParameterConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        ConversionResult::new(LanguageFunctionParameter {
            name: self.name,
            param_type: self.type_doc.unwrap_or_default(),
            default_value: self.default,
        })
    }

    fn from_ir(input: Self::IrType, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let type_doc = if input.param_type.is_empty() {
            None
        } else {
            Some(input.param_type)
        };
        ConversionResult::new(JsParameter {
            name: input.name,
            default: input.default_value,
            type_doc,
        })
    }
}

/// Conversion options for JavaScript parameters.
#[derive(Debug, Clone, Default)]
pub struct JsParameterConversionOptions {
    /// Cross-language conversion configuration.
    pub config: ConversionConfig,
}

/// Represents a JavaScript function or method.
///
/// The signature is always untyped. Any type information lives only in `param`
/// `type_doc` fields and `return_type`, which feed an optional JSDoc block and are
/// never rendered in the signature.
#[derive(Debug, Clone, PartialEq)]
pub struct JsFunction {
    /// The name of the function.
    pub name: String,
    /// The parameters of the function.
    pub parameters: Vec<JsParameter>,
    /// The return type for JSDoc only (e.g. `number`); never emitted in the signature.
    pub return_type: Option<String>,
    /// Optional free-form JSDoc description text, rendered as the first line of the block.
    pub doc: Option<String>,
    /// `true` to render the method with a leading `static` keyword.
    pub is_static: bool,
    /// The function body, one entry per line; `None` renders an empty `{}` block.
    pub body: Option<Vec<String>>,
}

impl BackendItem for JsFunction {
    type IrType = LanguageFunction;
    type ConversionOptions = JsFunctionConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut log = ConversionLog::new();

        let mut parameters = Vec::with_capacity(self.parameters.len());
        for parameter in self.parameters {
            let result = parameter.to_ir(None);
            log.add_warnings(result.log.warnings);
            parameters.push(result.value);
        }

        ConversionResult::with_log(
            LanguageFunction {
                name: self.name,
                visibility: Visibility::Public,
                parameters,
                generic_args: Vec::new(),
                return_type: self.return_type,
                is_static: self.is_static,
                is_abstract: false,
                is_virtual: false,
                is_override: false,
                is_final: false,
                body: self.body,
                docs: self.doc.map(|doc| vec![doc]),
                annotations: Vec::new(),
                raw_attributes: Vec::new(),
            },
            log,
        )
    }

    fn from_ir(input: Self::IrType, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();

        let mut parameters = Vec::with_capacity(input.parameters.len());
        for parameter in input.parameters {
            let result = JsParameter::from_ir(parameter, None);
            log.add_warnings(result.log.warnings);
            parameters.push(result.value);
        }

        ConversionResult::with_log(
            JsFunction {
                name: input.name,
                parameters,
                return_type: input.return_type,
                doc: input.docs.map(|docs| docs.join("\n")),
                is_static: input.is_static,
                body: input.body,
            },
            log,
        )
    }
}

/// Conversion options for JavaScript functions.
#[derive(Debug, Clone, Default)]
pub struct JsFunctionConversionOptions {
    /// Cross-language conversion configuration.
    pub config: ConversionConfig,
}

/// Render options for JavaScript functions.
#[derive(Debug, Clone)]
pub struct JsFunctionRenderOptions {
    /// Whether to render a JSDoc block when type information is present.
    ///
    /// JSDoc is emitted only when genuine type information exists on the model
    /// (any param `type_doc`, a `return_type`, or a `doc` string). When no such
    /// information exists, no JSDoc is synthesised regardless of this flag.
    pub render_jsdoc: bool,
}

impl Default for JsFunctionRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl JsFunctionRenderOptions {
    pub const DEFAULT: Self = Self { render_jsdoc: true };
}
