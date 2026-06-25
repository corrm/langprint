use super::Visibility;

/// Represents a constant in a language-agnostic way.
#[derive(Debug, Clone, PartialEq)]
pub struct LanguageConstant {
    /// The name of the constant.
    pub name: String,
    /// The visibility of the constant.
    pub visibility: Visibility,
    /// The data type of the constant.
    pub data_type: String,
    /// The value of the constant.
    pub value: String,
    /// Optional documentation for the constant.
    pub docs: Option<Vec<String>>,
}
