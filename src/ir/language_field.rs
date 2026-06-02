//! Field type used in structs and classes.

use super::Visibility;

/// Represents a field in a struct or class.
#[derive(Debug, Clone, PartialEq)]
pub struct LanguageField {
    /// The name of the field.
    pub name: String,
    /// The type of the field.
    pub field_type: String,
    /// The visibility of the field.
    pub visibility: Visibility,
    /// The size of the array if the field is an array.
    pub array_size: Option<String>,
    /// Bit field size (can be a number or a macro/define name).
    pub bit_field_size: Option<String>,
    /// Over-alignment for this field (`alignas(N)`); `None` = natural alignment.
    pub alignment: Option<u32>,
    /// Optional initialization value for the field.
    pub initialization_value: Option<String>,
    /// Inline comment for the field.
    pub inline_comment: Option<String>,
    /// Documentation for the field.
    pub docs: Option<Vec<String>>,
}
