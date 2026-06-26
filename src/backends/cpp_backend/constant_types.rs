use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult},
    ir::LanguageConstant,
};

use super::CppVisibility;

/// Represents a C++ constant.
#[derive(Debug, Clone, PartialEq)]
pub struct CppConstant {
    /// The name of the constant.
    pub name: String,
    /// The visibility of the constant.
    pub visibility: CppVisibility,
    /// The data type of the constant.
    pub data_type: String,
    /// The value of the constant.
    pub value: String,
    /// Optional documentation for the constant.
    pub docs: Option<Vec<String>>,
}

impl BackendItem for CppConstant {
    type IrType = LanguageConstant;
    type ConversionOptions = CppConstantConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        ConversionResult::new(LanguageConstant {
            name: self.name,
            visibility: self.visibility.to_ir(None).value,
            data_type: self.data_type,
            value: self.value,
            docs: self.docs,
        })
    }

    fn from_ir(
        input: Self::IrType,
        _options: Option<&Self::ConversionOptions>,
    ) -> ConversionResult<Self> {
        let mut result_log = ConversionLog::new();

        let visibility: ConversionResult<CppVisibility> =
            CppVisibility::from_ir(input.visibility, None);
        if visibility.log.has_warnings() {
            result_log.add_warnings(visibility.log.warnings);
        }

        ConversionResult::with_log(
            CppConstant {
                name: input.name,
                visibility: visibility.value,
                data_type: input.data_type,
                value: input.value,
                docs: input.docs,
            },
            result_log,
        )
    }
}

/// Conversion options for C++ constants.
#[derive(Debug, Clone)]
pub struct CppConstantConversionOptions {}

impl Default for CppConstantConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CppConstantConversionOptions {
    pub const DEFAULT: Self = Self {};
}

/// Render options for C++ constants.
#[derive(Debug, Clone)]
pub struct CppConstantRenderOptions {
    /// Whether to use constexpr instead of const.
    pub use_constexpr: bool,
    /// Whether to include inline specifier for constants.
    pub use_inline: bool,
    /// Whether to render documentation comments.
    pub render_docs: bool,
}

impl Default for CppConstantRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CppConstantRenderOptions {
    pub const DEFAULT: Self = Self {
        use_constexpr: false,
        use_inline: false,
        render_docs: true,
    };
}
