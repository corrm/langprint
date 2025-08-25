/// Represents a C++ template parameter.
#[derive(Debug, Clone)]
pub struct CppGenericArgument {
    /// The name of the template parameter.
    pub name: String,
    /// The keyword that precedes the parameter name.
    /// For C++, this is typically:
    /// - "typename" or "class" for type parameters
    /// - "int", "size_t", etc. for value parameters
    /// - "" (empty string) for parameters without an explicit keyword
    pub keyword: String,
    /// Optional default value for the template parameter.
    /// For example: "int" -> "T = int" or "10" -> "T = 10" in C++
    pub default_value: Option<String>,
}
