/// Represents different visibility levels across programming languages.
///
/// This enum abstracts common visibility concepts found in many languages,
/// while providing flexible options for module and scope-specific visibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    /// Default visibility, used when no visibility is specified.
    Default,

    /// Visible everywhere, no restrictions.
    Public,

    /// Visible to subclasses or derived classes.
    Protected,

    /// Visible only within the current scope, class, or module.
    Private,

    /// Visible within the immediate parent or enclosing module.
    ///
    /// Example:
    /// - Rust: `pub(super)`
    ParentModule,

    /// Visible within a specific named scope, module, or submodule.
    ///
    /// For languages supporting hierarchical modules or folders.
    ///
    /// Example:
    /// - Rust: `pub(in submodule)`
    Scoped(String),

    /// Visible within a package, crate, assembly, or similar unit.
    ///
    /// Examples:
    /// - Rust: `pub(crate)`
    /// - Java: package-private
    /// - C#: internal
    Package,

    /// Visible within a broader namespace or logical grouping.
    ///
    /// This is similar to `Package` but can represent larger or different
    /// logical units depending on language semantics.
    ///
    /// Examples:
    /// - C++: namespace
    /// - C#: namespace
    Namespace,
}
