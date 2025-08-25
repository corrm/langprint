use super::{LanguageConstant, LanguageDefinition, LanguageEnum, LanguageStruct, Visibility};

/// Represents a constant in a language-agnostic way.
#[derive(Debug, Clone)]
pub struct LanguageNamespace {
    /// The name of the constant.
    pub name: String,
    /// The visibility of the constant.
    pub visibility: Visibility,
    /// The defines in the namespace.
    pub defines: Option<Vec<LanguageDefinition>>,
    /// The constants in the namespace.
    pub constants: Option<Vec<LanguageConstant>>,
    /// The enums in the namespace.
    pub enums: Option<Vec<LanguageEnum>>,
    /// The structs in the namespace.
    pub structs: Option<Vec<LanguageStruct>>,
    /// The namespaces in the namespace.
    pub namespaces: Option<Vec<LanguageNamespace>>,
    /// Optional documentation for the constant.
    pub docs: Option<Vec<String>>,
}
