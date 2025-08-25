use super::{LanguageField, LanguageFunction, LanguageGenericArgument, Visibility};

/// Represents a base/super class or struct with its visibility.
#[derive(Debug, Clone)]
pub struct LanguageBase {
    /// The name of the base/super.
    pub name: String,
    /// The visibility of the inheritance.
    pub visibility: Visibility,
}

/// Represents a struct/class in a language-agnostic way.
#[derive(Debug, Clone)]
pub struct LanguageStruct {
    /// The visibility of the struct/class.
    pub visibility: Visibility,
    /// Whether the struct/class is a class.
    pub is_class: bool,
    /// Whether the struct/class is abstract.
    pub is_abstract: bool,
    /// Whether the struct/class is final.
    pub is_final: bool,
    /// The name of the struct/class.
    pub name: String,
    /// Generic arguments for the struct/class (e.g., template parameters in C++).
    pub generic_args: Vec<LanguageGenericArgument>,
    /// Base/super classes or structs that this struct/class inherits from.
    pub bases: Vec<LanguageBase>,
    /// The fields of the struct/class.
    pub fields: Vec<LanguageField>,
    /// The methods of the struct/class.
    pub methods: Vec<LanguageFunction>,
    /// Documentation for the struct/class.
    pub docs: Option<Vec<String>>,
}
