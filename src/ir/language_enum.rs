use super::Visibility;

/// Represents different kinds of enum variants.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub value: EnumVariantValue,
    pub docs: Option<Vec<String>>,
}

/// Represents different kinds of enum variants.
#[derive(Debug, Clone, PartialEq)]
pub enum EnumVariantValue {
    /// No value variant.
    NoValue,
    /// Value variant with a name and explicit value.
    Value(String),
    /// Tuple variant with a name and field types.
    Tuple(Vec<String>),
    /// Struct variant with a name and named fields with types.
    Struct(Vec<(String, String)>),
}

/// Represents an enum in a language-agnostic way.
#[derive(Debug, Clone, PartialEq)]
pub struct LanguageEnum {
    /// The name of the enum.
    pub name: String,
    /// The visibility of the enum.
    pub visibility: Visibility,
    /// The variants of the enum with optional documentation.
    pub variants: Vec<EnumVariant>,
    /// The underlying type of the enum (e.g., 'u8', 'i32', etc.).
    pub underlying_type: Option<String>,
    /// Documentation for the enum.
    pub docs: Option<Vec<String>>,
}
