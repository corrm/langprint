//! Field type used in structs and classes.

use super::{Annotation, RawAttribute, Visibility};

/// Represents a field in a struct or class.
#[derive(Debug, Clone, PartialEq)]
pub struct LanguageField {
    /// The name of the field.
    pub name: String,
    /// The type of the field.
    pub field_type: String,
    /// The visibility of the field.
    pub visibility: Visibility,
    /// Whether the field is associated with the type rather than an instance.
    pub is_static: bool,
    /// Whether the field is immutable.
    pub is_const: bool,
    /// Documentation for the field.
    pub docs: Option<Vec<String>>,
    /// Curated source-neutral annotations (Tier 1).
    pub annotations: Vec<Annotation>,
    /// Opaque source-tagged attributes carried verbatim (Tier 2).
    pub raw_attributes: Vec<RawAttribute>,
}
