/// Represents a define in a language-agnostic way.
#[derive(Debug, Clone, PartialEq)]
pub struct LanguageDefinition {
    /// The name of the define.
    pub name: String,
    /// The value of the define.
    pub value: Option<String>,
    /// Optional documentation for the define.
    pub docs: Option<Vec<String>>,
}
