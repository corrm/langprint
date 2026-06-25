use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    convert::{ConversionConfig, IdentifierKind, map_type, rename_identifier},
    ir::{LanguageFunction, LanguageFunctionParameter},
    type_map::{PrimitiveType, TargetLanguage},
};

use super::{CSharpGenericArgument, CSharpVisibility};

/// Represents a C# method parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct CSharpParameter {
    /// The name of the parameter.
    pub name: String,
    /// The type of the parameter.
    pub param_type: String,
    /// Default value for the parameter, if any.
    pub default_value: Option<String>,
}

impl BackendItem for CSharpParameter {
    type IrType = LanguageFunctionParameter;
    type ConversionOptions = CSharpParameterConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        ConversionResult::new(LanguageFunctionParameter {
            name: self.name,
            param_type: self.param_type,
            default_value: self.default_value,
        })
    }

    fn from_ir(input: Self::IrType, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let config = options.map(|options| options.config.clone()).unwrap_or_default();
        let param_type = map_type(&config, &input.param_type, TargetLanguage::CSharp);

        ConversionResult::with_log(
            CSharpParameter {
                name: input.name,
                param_type: param_type.value,
                default_value: input.default_value,
            },
            param_type.log,
        )
    }
}

/// Conversion options for C# parameters.
#[derive(Debug, Clone, Default)]
pub struct CSharpParameterConversionOptions {
    /// Cross-language conversion configuration (type mapping + renaming).
    pub config: ConversionConfig,
}

/// Represents a C# method.
#[derive(Debug, Clone, PartialEq)]
pub struct CSharpMethod {
    /// The name of the method.
    pub name: String,
    /// The visibility of the method.
    pub visibility: CSharpVisibility,
    /// The parameters of the method.
    pub parameters: Vec<CSharpParameter>,
    /// Generic type parameters of the method.
    pub generic_args: Vec<CSharpGenericArgument>,
    /// The return type of the method (`None` renders as `void`).
    pub return_type: Option<String>,
    /// Whether the method is `static`.
    pub is_static: bool,
    /// Whether the method is `abstract`.
    pub is_abstract: bool,
    /// Whether the method is `virtual`.
    pub is_virtual: bool,
    /// Whether the method is `override`.
    pub is_override: bool,
    /// Whether the method is `sealed`.
    pub is_sealed: bool,
    /// Whether the method is `async`.
    pub is_async: bool,
    /// Whether the method is `unsafe`.
    pub is_unsafe: bool,
    /// The method body, one entry per line; `None` renders an abstract/interface declaration.
    pub body: Option<Vec<String>>,
    /// Attributes applied to the method (without the leading `[`).
    pub attributes: Vec<String>,
    /// Documentation for the method.
    pub docs: Option<Vec<String>>,
}

impl BackendItem for CSharpMethod {
    type IrType = LanguageFunction;
    type ConversionOptions = CSharpMethodConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut log = ConversionLog::new();

        if self.is_async {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("`async` on method `{}`", self.name),
                resolution: "async modifier dropped from the language-agnostic IR".to_string(),
            });
        }
        if self.is_unsafe {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("`unsafe` on method `{}`", self.name),
                resolution: "unsafe modifier dropped from the language-agnostic IR".to_string(),
            });
        }
        for attribute in &self.attributes {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("attribute `[{}]` on method `{}`", attribute, self.name),
                resolution: "C# attributes dropped from the language-agnostic IR".to_string(),
            });
        }

        let visibility = self.visibility.to_ir(None);
        log.add_warnings(visibility.log.warnings);

        let mut parameters = Vec::with_capacity(self.parameters.len());
        for param in self.parameters {
            let result = param.to_ir(None);
            log.add_warnings(result.log.warnings);
            parameters.push(result.value);
        }

        let mut generic_args = Vec::with_capacity(self.generic_args.len());
        for generic in self.generic_args {
            let result = generic.to_ir(None);
            log.add_warnings(result.log.warnings);
            generic_args.push(result.value);
        }

        let function = LanguageFunction {
            name: self.name,
            visibility: visibility.value,
            parameters,
            generic_args,
            return_type: self.return_type,
            is_static: self.is_static,
            is_abstract: self.is_abstract,
            is_virtual: self.is_virtual,
            is_override: self.is_override,
            is_final: self.is_sealed,
            body: self.body,
            docs: self.docs,
        };
        ConversionResult::with_log(function, log)
    }

    fn from_ir(input: Self::IrType, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();
        let config = options.map(|options| options.config.clone()).unwrap_or_default();

        let name = rename_identifier(&config, &input.name, TargetLanguage::CSharp, IdentifierKind::Function);
        log.add_warnings(name.log.warnings);

        let visibility = CSharpVisibility::from_ir(input.visibility, None);
        log.add_warnings(visibility.log.warnings);

        let parameter_options = CSharpParameterConversionOptions { config: config.clone() };
        let mut parameters = Vec::with_capacity(input.parameters.len());
        for param in input.parameters {
            let result = CSharpParameter::from_ir(param, Some(&parameter_options));
            log.add_warnings(result.log.warnings);
            parameters.push(result.value);
        }

        let mut generic_args = Vec::with_capacity(input.generic_args.len());
        for generic in &input.generic_args {
            let result = CSharpGenericArgument::from_ir(generic.clone(), None);
            log.add_warnings(result.log.warnings);
            generic_args.push(result.value);
        }

        // A `void`/unit return is the absence of a return type in C# (`None` renders `void`).
        let return_type = match input.return_type {
            Some(return_type) if config.type_map.resolve(&return_type) == Some(PrimitiveType::Void) => None,
            Some(return_type) => {
                let mapped = map_type(&config, &return_type, TargetLanguage::CSharp);
                log.add_warnings(mapped.log.warnings);
                Some(mapped.value)
            }
            None => None,
        };

        let method = CSharpMethod {
            name: name.value,
            visibility: visibility.value,
            parameters,
            generic_args,
            return_type,
            is_static: input.is_static,
            is_abstract: input.is_abstract,
            is_virtual: input.is_virtual,
            is_override: input.is_override,
            is_sealed: input.is_final,
            is_async: false,
            is_unsafe: false,
            body: input.body,
            attributes: Vec::new(),
            docs: input.docs,
        };
        ConversionResult::with_log(method, log)
    }
}

/// Conversion options for C# methods.
#[derive(Debug, Clone, Default)]
pub struct CSharpMethodConversionOptions {
    /// Cross-language conversion configuration (type mapping + renaming).
    pub config: ConversionConfig,
}

/// Render options for C# methods.
#[derive(Debug, Clone)]
pub struct CSharpMethodRenderOptions {
    /// Whether to render documentation comments.
    pub render_docs: bool,
    /// Whether to render attributes.
    pub render_attributes: bool,
}

impl Default for CSharpMethodRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CSharpMethodRenderOptions {
    pub const DEFAULT: Self = Self {
        render_docs: true,
        render_attributes: true,
    };
}
