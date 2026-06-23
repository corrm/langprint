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
    /// Whether the function is associated with the type rather than an instance.
    pub is_static: bool,
    /// Whether the function has no implementation and must be provided by an implementor.
    pub is_abstract: bool,
    /// Whether the function can be overridden by a derived type.
    pub is_virtual: bool,
    /// Whether the function overrides a base type method.
    pub is_override: bool,
    /// Whether the function cannot be further overridden.
    pub is_final: bool,
    /// The function body code, if available. Each string represents a line of code.
    pub body: Option<Vec<String>>,
    /// Documentation for the function.
    pub docs: Option<Vec<String>>,
}
