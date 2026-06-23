use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    ir::{LanguageFunction, LanguageFunctionParameter},
};

use super::{RustGenericArgument, RustVisibility};

/// The receiver of a Rust method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustSelfKind {
    /// A free function or associated function — no receiver.
    None,
    /// `&self`.
    Ref,
    /// `&mut self`.
    RefMut,
    /// `self` (by value).
    Owned,
}

impl RustSelfKind {
    /// Render the receiver as it appears in a parameter list, or `None` when there is no receiver.
    pub(crate) fn render(&self) -> Option<&'static str> {
        match self {
            RustSelfKind::None => None,
            RustSelfKind::Ref => Some("&self"),
            RustSelfKind::RefMut => Some("&mut self"),
            RustSelfKind::Owned => Some("self"),
        }
    }
}

/// Represents a parameter of a Rust function.
#[derive(Debug, Clone)]
pub struct RustParameter {
    /// The name of the parameter.
    pub name: String,
    /// The type of the parameter.
    pub param_type: String,
}

impl BackendItem for RustParameter {
    type IrType = LanguageFunctionParameter;
    type ConversionOptions = RustParameterConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        ConversionResult::new(LanguageFunctionParameter {
            name: self.name,
            param_type: self.param_type,
            default_value: None,
        })
    }

    fn from_ir(input: Self::IrType, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();

        if input.default_value.is_some() {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("default value on parameter `{}`", input.name),
                resolution: "Rust has no default parameters; the default was dropped".to_string(),
            });
        }

        ConversionResult::with_log(
            RustParameter {
                name: input.name,
                param_type: input.param_type,
            },
            log,
        )
    }
}

/// Conversion options for Rust parameters.
#[derive(Debug, Clone)]
pub struct RustParameterConversionOptions {}

impl Default for RustParameterConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl RustParameterConversionOptions {
    pub const DEFAULT: Self = Self {};
}

/// Represents a Rust function or method.
#[derive(Debug, Clone)]
pub struct RustFunction {
    /// The name of the function.
    pub name: String,
    /// The visibility of the function.
    pub visibility: RustVisibility,
    /// The receiver of the function (for methods).
    pub self_kind: RustSelfKind,
    /// The parameters of the function (excluding the receiver).
    pub parameters: Vec<RustParameter>,
    /// Generic parameters of the function.
    pub generic_args: Vec<RustGenericArgument>,
    /// The return type of the function; `None` for `()`.
    pub return_type: Option<String>,
    /// Whether the function is `unsafe`.
    pub is_unsafe: bool,
    /// Whether the function is `async`.
    pub is_async: bool,
    /// Whether the function is `const`.
    pub is_const: bool,
    /// The function body, one entry per line; `None` renders a bare signature (declaration only).
    pub body: Option<Vec<String>>,
    /// Attributes applied to the function (without the leading `#[`).
    pub attributes: Vec<String>,
    /// Optional documentation for the function.
    pub docs: Option<Vec<String>>,
}

impl BackendItem for RustFunction {
    type IrType = LanguageFunction;
    type ConversionOptions = RustFunctionConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut log = ConversionLog::new();

        if self.is_unsafe {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("`unsafe` on function `{}`", self.name),
                resolution: "the `unsafe` qualifier is dropped from the language-agnostic IR".to_string(),
            });
        }
        if self.is_async {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("`async` on function `{}`", self.name),
                resolution: "the `async` qualifier is dropped from the language-agnostic IR".to_string(),
            });
        }
        if self.is_const {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("`const` on function `{}`", self.name),
                resolution: "the `const fn` qualifier is dropped from the language-agnostic IR".to_string(),
            });
        }
        for attribute in &self.attributes {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("attribute `#[{}]` on function `{}`", attribute, self.name),
                resolution: "Rust attributes are dropped from the language-agnostic IR".to_string(),
            });
        }
        if matches!(self.self_kind, RustSelfKind::RefMut | RustSelfKind::Owned) {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!(
                    "`{}` receiver on method `{}`",
                    self.self_kind.render().unwrap_or("self"),
                    self.name
                ),
                resolution: "the IR carries only instance-vs-static; lowered to `&self`".to_string(),
            });
        }

        let visibility = self.visibility.to_ir(None);
        log.add_warnings(visibility.log.warnings);

        let mut parameters = Vec::with_capacity(self.parameters.len());
        for parameter in self.parameters {
            let result = parameter.to_ir(None);
            log.add_warnings(result.log.warnings);
            parameters.push(result.value);
        }

        let generic_args = self.generic_args.iter().map(RustGenericArgument::to_ir).collect();

        ConversionResult::with_log(
            LanguageFunction {
                name: self.name,
                visibility: visibility.value,
                parameters,
                generic_args,
                return_type: self.return_type,
                is_static: self.self_kind == RustSelfKind::None,
                is_abstract: false,
                is_virtual: false,
                is_override: false,
                is_final: false,
                body: self.body,
                docs: self.docs,
            },
            log,
        )
    }

    fn from_ir(input: Self::IrType, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();

        if input.is_virtual {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("virtual method `{}`", input.name),
                resolution: "Rust has no virtual methods; lowered to an inherent method".to_string(),
            });
        }
        if input.is_override {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("override method `{}`", input.name),
                resolution: "Rust has no method overriding; the modifier was dropped".to_string(),
            });
        }
        if input.is_final {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("final method `{}`", input.name),
                resolution: "Rust methods are not overridable; `final` was dropped".to_string(),
            });
        }
        if input.is_abstract {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("abstract method `{}`", input.name),
                resolution: "lowered to a bare method signature (declaration only)".to_string(),
            });
        }

        let visibility = RustVisibility::from_ir(input.visibility, None);
        log.add_warnings(visibility.log.warnings);

        let mut parameters = Vec::with_capacity(input.parameters.len());
        for parameter in input.parameters {
            let result = RustParameter::from_ir(parameter, None);
            log.add_warnings(result.log.warnings);
            parameters.push(result.value);
        }

        let generic_args = input.generic_args.iter().map(RustGenericArgument::from_ir).collect();

        let self_kind = if input.is_static {
            RustSelfKind::None
        } else {
            RustSelfKind::Ref
        };

        ConversionResult::with_log(
            RustFunction {
                name: input.name,
                visibility: visibility.value,
                self_kind,
                parameters,
                generic_args,
                return_type: input.return_type,
                is_unsafe: false,
                is_async: false,
                is_const: false,
                body: input.body,
                attributes: Vec::new(),
                docs: input.docs,
            },
            log,
        )
    }
}

/// Conversion options for Rust functions.
#[derive(Debug, Clone)]
pub struct RustFunctionConversionOptions {}

impl Default for RustFunctionConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl RustFunctionConversionOptions {
    pub const DEFAULT: Self = Self {};
}

/// Render options for Rust functions.
#[derive(Debug, Clone)]
pub struct RustFunctionRenderOptions {
    /// Whether to render documentation comments.
    pub render_docs: bool,
    /// Whether to render attributes.
    pub render_attributes: bool,
}

impl Default for RustFunctionRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl RustFunctionRenderOptions {
    pub const DEFAULT: Self = Self {
        render_docs: true,
        render_attributes: true,
    };
}
