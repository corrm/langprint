use super::CSharpVisibility;

/// Represents a C# property.
///
/// A property is a C#-only enrichment with no direct neutral-IR counterpart: when a
/// [`CSharpType`](super::CSharpType) is projected to the IR its properties are lowered to
/// fields (with a [`ConversionWarning`](crate::conversion::ConversionWarning)). `from_ir`
/// never produces a property — it produces fields — so [`CSharpProperty`] is render-only and
/// is constructed directly by callers who want idiomatic C# properties.
#[derive(Debug, Clone, PartialEq)]
pub struct CSharpProperty {
    /// The name of the property.
    pub name: String,
    /// The type of the property.
    pub prop_type: String,
    /// The visibility of the property.
    pub visibility: CSharpVisibility,
    /// Whether the property is `static`.
    pub is_static: bool,
    /// Whether the property has a getter.
    pub has_getter: bool,
    /// Whether the property has a setter.
    pub has_setter: bool,
    /// Optional getter body, one entry per line; `None` renders an auto-accessor (`get;`).
    pub getter_body: Option<Vec<String>>,
    /// Optional setter body, one entry per line; `None` renders an auto-accessor (`set;`).
    pub setter_body: Option<Vec<String>>,
    /// Documentation for the property.
    pub docs: Option<Vec<String>>,
}
