use crate::{backends::BackendItem, conversion::ConversionResult, ir::LanguageConstant};

use super::CSharpVisibility;

/// Represents a C# constant (`const` field).
#[derive(Debug, Clone, PartialEq)]
pub struct CSharpConstant {
    /// The name of the constant.
    pub name: String,
    /// The visibility of the constant.
    pub visibility: CSharpVisibility,
    /// The data type of the constant.
    pub data_type: String,
    /// The value of the constant.
    pub value: String,
    /// Optional documentation for the constant.
    pub docs: Option<Vec<String>>,
}

impl BackendItem for CSharpConstant {
    type IrType = LanguageConstant;
    type ConversionOptions = CSharpConstantConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let visibility = self.visibility.to_ir(None);
        let constant = LanguageConstant {
            name: self.name,
            visibility: visibility.value,
            data_type: self.data_type,
            value: self.value,
            docs: self.docs,
        };
        ConversionResult::with_log(constant, visibility.log)
    }

    fn from_ir(
        input: Self::IrType,
        _options: Option<&Self::ConversionOptions>,
    ) -> ConversionResult<Self> {
        let visibility = CSharpVisibility::from_ir(input.visibility, None);
        let constant = CSharpConstant {
            name: input.name,
            visibility: visibility.value,
            data_type: input.data_type,
            value: input.value,
            docs: input.docs,
        };
        ConversionResult::with_log(constant, visibility.log)
    }
}

/// Conversion options for C# constants.
#[derive(Debug, Clone)]
pub struct CSharpConstantConversionOptions {}

impl Default for CSharpConstantConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CSharpConstantConversionOptions {
    pub const DEFAULT: Self = Self {};
}

/// Render options for C# constants.
#[derive(Debug, Clone)]
pub struct CSharpConstantRenderOptions {
    /// Whether to render documentation comments.
    pub render_docs: bool,
}

impl Default for CSharpConstantRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CSharpConstantRenderOptions {
    pub const DEFAULT: Self = Self { render_docs: true };
}
