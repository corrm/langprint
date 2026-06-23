use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    ir::LanguageConstant,
};

use super::RustVisibility;

/// Represents a Rust constant (`const`) or `static` item.
#[derive(Debug, Clone)]
pub struct RustConstant {
    /// The name of the constant.
    pub name: String,
    /// The visibility of the constant.
    pub visibility: RustVisibility,
    /// The type of the constant.
    pub data_type: String,
    /// The value expression of the constant.
    pub value: String,
    /// Whether to emit `static` instead of `const`.
    pub is_static: bool,
    /// Optional documentation for the constant.
    pub docs: Option<Vec<String>>,
}

impl BackendItem for RustConstant {
    type IrType = LanguageConstant;
    type ConversionOptions = RustConstantConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut log = ConversionLog::new();

        if self.is_static {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("`static` item `{}`", self.name),
                resolution: "lowered to a plain constant in the language-agnostic IR".to_string(),
            });
        }

        let visibility = self.visibility.to_ir(None);
        log.add_warnings(visibility.log.warnings);

        ConversionResult::with_log(
            LanguageConstant {
                name: self.name,
                visibility: visibility.value,
                data_type: self.data_type,
                value: self.value,
                docs: self.docs,
            },
            log,
        )
    }

    fn from_ir(input: Self::IrType, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();

        let visibility = RustVisibility::from_ir(input.visibility, None);
        log.add_warnings(visibility.log.warnings);

        ConversionResult::with_log(
            RustConstant {
                name: input.name,
                visibility: visibility.value,
                data_type: input.data_type,
                value: input.value,
                is_static: false,
                docs: input.docs,
            },
            log,
        )
    }
}

/// Conversion options for Rust constants.
#[derive(Debug, Clone)]
pub struct RustConstantConversionOptions {}

impl Default for RustConstantConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl RustConstantConversionOptions {
    pub const DEFAULT: Self = Self {};
}

/// Render options for Rust constants.
#[derive(Debug, Clone)]
pub struct RustConstantRenderOptions {
    /// Whether to render documentation comments.
    pub render_docs: bool,
}

impl Default for RustConstantRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl RustConstantRenderOptions {
    pub const DEFAULT: Self = Self { render_docs: true };
}
