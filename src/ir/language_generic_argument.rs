/// Represents a generic argument for a function or class.
#[derive(Debug, Clone, PartialEq)]
pub struct LanguageGenericArgument {
    /// The name of the generic argument.
    pub name: String,
    /// The keyword or specifier that defines this generic parameter's nature.
    /// Language-specific examples:
    /// - C++: "typename", "class", "int", "size_t", etc.
    /// - Rust: empty string (no explicit keyword) or specific type for const generics
    /// - C#: empty string (no explicit keyword)
    /// - Java: empty string (no explicit keyword)
    pub keyword: String,
    /// Optional default value for the generic parameter.
    /// Language-specific examples:
    /// - C++: "int" -> "T = int" or "10" -> "T = 10"
    /// - Java: "Number" -> "T extends Number"
    /// - C#: "IComparable" -> "T where T : IComparable"
    pub default_value: Option<String>,
    /// Optional where clause or constraint for the generic parameter.
    /// Language-specific examples:
    /// - C#: "IComparable" -> "where T : IComparable"
    /// - Rust: "Display + Debug" -> "where T: Display + Debug"
    /// - Java: "extends Number" or "implements Comparable"
    pub where_clause: Option<String>,
}
