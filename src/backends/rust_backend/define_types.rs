use crate::{backends::BackendItem, conversion::ConversionResult, ir::LanguageDefinition};

/// Represents a Rust lowering of a preprocessor-style define.
///
/// Rust has no preprocessor, so a C/C++ `#define` is lowered to a module-level constant.
/// A define with no value becomes a unit constant (`pub const NAME: () = ();`), preserving the
/// "is defined" marker.
#[derive(Debug, Clone, PartialEq)]
pub struct RustDefinition {
    /// The name of the define.
    pub name: String,
    /// The value expression, if any.
    pub value: Option<String>,
    /// Optional documentation for the define.
    pub docs: Option<Vec<String>>,
}

impl BackendItem for RustDefinition {
    type IrType = LanguageDefinition;
    type ConversionOptions = RustDefinitionConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        ConversionResult::new(LanguageDefinition {
            name: self.name,
            value: self.value,
            docs: self.docs,
        })
    }

    fn from_ir(input: Self::IrType, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        ConversionResult::new(RustDefinition {
            name: input.name,
            value: input.value,
            docs: input.docs,
        })
    }
}

/// Conversion options for Rust definitions.
#[derive(Debug, Clone)]
pub struct RustDefinitionConversionOptions {}

impl Default for RustDefinitionConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl RustDefinitionConversionOptions {
    pub const DEFAULT: Self = Self {};
}

/// Render options for Rust definitions.
#[derive(Debug, Clone)]
pub struct RustDefinitionRenderOptions {
    /// The constant type to emit for a value-carrying define (defines are untyped in C).
    pub const_type: String,
    /// Whether to render documentation comments.
    pub render_docs: bool,
}

impl Default for RustDefinitionRenderOptions {
    fn default() -> Self {
        Self {
            const_type: "i64".to_string(),
            render_docs: true,
        }
    }
}
