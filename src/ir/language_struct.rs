use super::{LanguageField, LanguageFunction, LanguageGenericArgument, Visibility};

/// Represents a base/super class or struct with its visibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageBase {
    /// The name of the base/super.
    pub name: String,
    /// The visibility of the inheritance.
    pub visibility: Visibility,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LanguageStructKind {
    Struct,
    Class,
    Union,
}

/// Represents a struct/class in a language-agnostic way.
#[derive(Debug, Clone, PartialEq)]
pub struct LanguageStruct {
    /// The visibility of the struct/class.
    pub visibility: Visibility,
    /// Struct kind.
    pub struct_kind: LanguageStructKind,
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
