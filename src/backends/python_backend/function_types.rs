use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult},
    convert::{map_type, rename_identifier, ConversionConfig, IdentifierKind},
    ir::{LanguageFunction, LanguageFunctionParameter, Visibility},
    type_map::{PrimitiveType, TargetLanguage},
};

/// Represents a parameter of a Python function.
///
/// Type hints and defaults are both optional and free-form: Python only carries
/// the annotations the source actually wrote, so they are honestly modelled as
/// `Option<String>` rather than synthesised.
#[derive(Debug, Clone, PartialEq)]
pub struct PythonParameter {
    /// The name of the parameter.
    pub name: String,
    /// The type hint of the parameter, if one was written (e.g. `int`).
    pub type_hint: Option<String>,
    /// The default value of the parameter, if any (e.g. `0`).
    pub default: Option<String>,
}

impl BackendItem for PythonParameter {
    type IrType = LanguageFunctionParameter;
    type ConversionOptions = PythonParameterConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        ConversionResult::new(LanguageFunctionParameter {
            name: self.name,
            param_type: self.type_hint.unwrap_or_default(),
            default_value: self.default,
        })
    }

    fn from_ir(input: Self::IrType, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();
        let config = options.map(|options| options.config.clone()).unwrap_or_default();

        let name = rename_identifier(&config, &input.name, TargetLanguage::Python, IdentifierKind::Field);
        log.add_warnings(name.log.warnings);

        let type_hint = if input.param_type.is_empty() {
            None
        } else {
            let mapped = map_type(&config, &input.param_type, TargetLanguage::Python);
            log.add_warnings(mapped.log.warnings);
            Some(mapped.value)
        };

        ConversionResult::with_log(
            PythonParameter {
                name: name.value,
                type_hint,
                default: input.default_value,
            },
            log,
        )
    }
}

/// Conversion options for Python parameters.
#[derive(Debug, Clone, Default)]
pub struct PythonParameterConversionOptions {
    /// Cross-language conversion configuration.
    pub config: ConversionConfig,
}

/// Represents a Python function or method (`def`).
#[derive(Debug, Clone, PartialEq)]
pub struct PythonFunction {
    /// The name of the function.
    pub name: String,
    /// The parameters of the function (including any `self`/`cls` receiver written verbatim).
    pub parameters: Vec<PythonParameter>,
    /// The return type hint, if one was written (e.g. `int`).
    pub return_type: Option<String>,
    /// Optional docstring, rendered as the first triple-quoted body line.
    pub docstring: Option<String>,
    /// The function body, one entry per line; `None` renders a single `pass` (declaration only).
    pub body: Option<Vec<String>>,
}

impl BackendItem for PythonFunction {
    type IrType = LanguageFunction;
    type ConversionOptions = PythonFunctionConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut log = ConversionLog::new();

        let mut parameters = Vec::with_capacity(self.parameters.len());
        for parameter in self.parameters {
            let result = parameter.to_ir(None);
            log.add_warnings(result.log.warnings);
            parameters.push(result.value);
        }

        let docs = match (self.docstring, self.body) {
            (Some(docstring), body) => {
                let mut docs = vec![docstring];
                if let Some(body) = body {
                    docs.extend(body);
                }
                Some(docs)
            }
            (None, body) => body,
        };

        ConversionResult::with_log(
            LanguageFunction {
                name: self.name,
                visibility: Visibility::Public,
                parameters,
                generic_args: Vec::new(),
                return_type: self.return_type,
                is_static: true,
                is_abstract: false,
                is_virtual: false,
                is_override: false,
                is_final: false,
                body: docs,
                docs: None,
                annotations: Vec::new(),
                raw_attributes: Vec::new(),
            },
            log,
        )
    }

    fn from_ir(input: Self::IrType, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();
        let config = options.map(|options| options.config.clone()).unwrap_or_default();

        let name = rename_identifier(&config, &input.name, TargetLanguage::Python, IdentifierKind::Function);
        log.add_warnings(name.log.warnings);

        let parameter_options = PythonParameterConversionOptions { config: config.clone() };
        let mut parameters = Vec::with_capacity(input.parameters.len());
        for parameter in input.parameters {
            let result = PythonParameter::from_ir(parameter, Some(&parameter_options));
            log.add_warnings(result.log.warnings);
            parameters.push(result.value);
        }

        // A `void` return carries no PEP-484 annotation; idiomatic Python omits it.
        let return_type = match input.return_type {
            Some(return_type) if config.type_map.resolve(&return_type) == Some(PrimitiveType::Void) => None,
            Some(return_type) => {
                let mapped = map_type(&config, &return_type, TargetLanguage::Python);
                log.add_warnings(mapped.log.warnings);
                Some(mapped.value)
            }
            None => None,
        };

        ConversionResult::with_log(
            PythonFunction {
                name: name.value,
                parameters,
                return_type,
                docstring: input.docs.map(|docs| docs.join("\n")),
                body: input.body,
            },
            log,
        )
    }
}

/// Conversion options for Python functions.
#[derive(Debug, Clone, Default)]
pub struct PythonFunctionConversionOptions {
    /// Cross-language conversion configuration.
    pub config: ConversionConfig,
}

/// Render options for Python functions.
#[derive(Debug, Clone)]
pub struct PythonFunctionRenderOptions {
    /// Whether to render the docstring.
    pub render_docstring: bool,
}

impl Default for PythonFunctionRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl PythonFunctionRenderOptions {
    pub const DEFAULT: Self = Self { render_docstring: true };
}
