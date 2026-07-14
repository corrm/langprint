use super::{
    LanguageConstant, LanguageDefinition, LanguageEnum, LanguageFunction, LanguageStruct,
    RawAttribute, Visibility,
};

/// Represents a namespace/module in a language-agnostic way.
#[derive(Debug, Clone, PartialEq)]
pub struct LanguageNamespace {
    /// The name of the namespace.
    pub name: String,
    /// The visibility of the namespace.
    pub visibility: Visibility,
    /// The defines in the namespace.
    pub defines: Option<Vec<LanguageDefinition>>,
    /// The constants in the namespace.
    pub constants: Option<Vec<LanguageConstant>>,
    /// The enums in the namespace.
    pub enums: Option<Vec<LanguageEnum>>,
    /// The structs in the namespace.
    pub structs: Option<Vec<LanguageStruct>>,
    /// The free functions in the namespace.
    pub functions: Option<Vec<LanguageFunction>>,
    /// The namespaces nested in this namespace.
    pub namespaces: Option<Vec<LanguageNamespace>>,
    /// Optional documentation for the namespace.
    pub docs: Option<Vec<String>>,
    /// Opaque source-tagged attributes applied to this module/namespace.
    ///
    /// Each entry is an inner attribute value. The target renderer owns the
    /// surrounding syntax.
    pub raw_attributes: Vec<RawAttribute>,
}
