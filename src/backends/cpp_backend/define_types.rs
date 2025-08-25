use crate::{backends::BackendItem, conversion::ConversionResult, ir::LanguageDefinition};

/// Represents a C++ define.
#[derive(Debug, Clone)]
pub struct CppDefinition {
    /// The name of the define.
    pub name: String,
    /// The value of the define.
    pub value: Option<String>,
    /// Documentation for the define.
    pub docs: Option<Vec<String>>,
}

impl BackendItem for CppDefinition {
    type IrType = LanguageDefinition;
    type ConversionOptions = CppDefinitionConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        ConversionResult::new(LanguageDefinition {
            name: self.name,
            value: self.value,
            docs: self.docs,
        })
    }

    fn from_ir(
        input: Self::IrType,
        _options: Option<&Self::ConversionOptions>,
    ) -> ConversionResult<Self> {
        ConversionResult::new(CppDefinition {
            name: input.name,
            value: input.value,
            docs: input.docs,
        })
    }
}

/// Conversion options for C++ defines.
#[derive(Debug, Clone)]
pub struct CppDefinitionConversionOptions {}

impl Default for CppDefinitionConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CppDefinitionConversionOptions {
    pub const DEFAULT: Self = Self {};
}

/// Render options for C++ defines.
#[derive(Debug, Clone)]
pub struct CppDefinitionRenderOptions {
    /// Whether to render documentation comments.
    pub render_docs: bool,
}

impl Default for CppDefinitionRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CppDefinitionRenderOptions {
    pub const DEFAULT: Self = Self { render_docs: true };
}
