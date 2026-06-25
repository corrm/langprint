use crate::{backends::BackendItem, conversion::ConversionResult, ir::LanguageDefinition};

/// Infer the C# `const` type for a define value.
///
/// # Arguments
///
/// * `value` - The define's value expression.
///
/// # Returns
///
/// The C# keyword type (`bool`/`string`/`char`/`int`/`long`/`double`) the literal denotes, or
/// `None` when the value is not a recognized literal (the caller falls back to a configured type).
pub fn infer_const_type(value: &str) -> Option<&'static str> {
    let value = value.trim();
    if value == "true" || value == "false" {
        return Some("bool");
    }
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        return Some("string");
    }
    if value.len() >= 2 && value.starts_with('\'') && value.ends_with('\'') {
        return Some("char");
    }
    if let Some(hex) = value.strip_prefix("0x").or_else(|| value.strip_prefix("0X"))
        && i64::from_str_radix(hex, 16).is_ok()
    {
        return Some("int");
    }
    if value.parse::<i32>().is_ok() {
        return Some("int");
    }
    if value.parse::<i64>().is_ok() {
        return Some("long");
    }
    if value.parse::<f64>().is_ok() {
        return Some("double");
    }
    None
}

/// Represents a preprocessor-style define lowered into C#.
///
/// C# has no value-carrying `#define`, so a define is rendered as a `public const`
/// (see [`CSharpDefinitionRenderOptions::const_type`]).
#[derive(Debug, Clone, PartialEq)]
pub struct CSharpDefinition {
    /// The name of the define.
    pub name: String,
    /// The value of the define, if any.
    pub value: Option<String>,
    /// Optional documentation for the define.
    pub docs: Option<Vec<String>>,
}

impl BackendItem for CSharpDefinition {
    type IrType = LanguageDefinition;
    type ConversionOptions = CSharpDefinitionConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        ConversionResult::new(LanguageDefinition {
            name: self.name,
            value: self.value,
            docs: self.docs,
        })
    }

    fn from_ir(input: Self::IrType, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        ConversionResult::new(CSharpDefinition {
            name: input.name,
            value: input.value,
            docs: input.docs,
        })
    }
}

/// Conversion options for C# defines.
#[derive(Debug, Clone)]
pub struct CSharpDefinitionConversionOptions {}

impl Default for CSharpDefinitionConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CSharpDefinitionConversionOptions {
    pub const DEFAULT: Self = Self {};
}

/// Render options for C# defines.
#[derive(Debug, Clone)]
pub struct CSharpDefinitionRenderOptions {
    /// The C# type used for the generated `const` when the define has a value.
    pub const_type: &'static str,
    /// Whether to render documentation comments.
    pub render_docs: bool,
}

impl Default for CSharpDefinitionRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CSharpDefinitionRenderOptions {
    pub const DEFAULT: Self = Self {
        const_type: "int",
        render_docs: true,
    };
}
