use super::{LanguageGenericArgument, Visibility};

/// Represents a function parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct LanguageFunctionParameter {
    /// The name of the parameter.
    pub name: String,
    /// The type of the parameter.
    pub param_type: String,
    /// Default value for the parameter, if any.
    pub default_value: Option<String>,
}

/// Represents a function in a language-agnostic way.
#[derive(Debug, Clone, PartialEq)]
pub struct LanguageFunction {
    /// The name of the function.
    pub name: String,
    /// The visibility of the function.
    pub visibility: Visibility,
    /// The parameters of the function.
    pub parameters: Vec<LanguageFunctionParameter>,
    /// Generic arguments for the function (e.g., template parameters in C++).
    pub generic_args: Vec<LanguageGenericArgument>,
    /// The return type of the function.
    pub return_type: Option<String>,
    /// Whether the function is static/class method.
    pub is_static: bool,
    /// Whether the function is const (doesn't modify state).
    pub is_const: bool,
    /// Whether the function is abstract/virtual.
    pub is_abstract: bool,
    /// Whether the function is virtual (C++ specific, but useful for other OOP languages).
    pub is_virtual: bool,
    /// Whether the function is pure virtual (C++ specific, but useful for other OOP languages).
    pub is_pure_virtual: bool,
    /// Whether the function is inline.
    pub is_inline: bool,
    /// Whether the function is noexcept (C++ specific).
    pub is_noexcept: bool,
    /// Whether the function overrides a base class method.
    pub is_override: bool,
    /// Whether the function is marked as final.
    pub is_final: bool,
    /// Whether the function is marked as friend.
    pub is_friend: bool,
    /// Whether the function is deleted (C++11 = delete).
    pub is_deleted: bool,
    /// Whether the function is defaulted (C++11 = default).
    pub is_default: bool,
    /// The function body code, if available. Each string represents a line of code.
    pub body: Option<Vec<String>>,
    /// Documentation for the function.
    pub docs: Option<Vec<String>>,
}
