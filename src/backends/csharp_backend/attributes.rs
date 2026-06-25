//! Mapping between C# native attribute strings and the neutral [`Annotation`] vocabulary.

use crate::ir::Annotation;

/// Recognize a C# attribute body (without the leading `[`) as a Tier-1 [`Annotation`].
///
/// Returns `None` when the attribute has no curated mapping; the caller carries it as a
/// [`crate::ir::RawAttribute`] instead.
pub fn csharp_attribute_to_annotation(attribute: &str) -> Option<Annotation> {
    match normalize(attribute).as_str() {
        "StructLayout(LayoutKind.Sequential)" => Some(Annotation::ReprC),
        "StructLayout(LayoutKind.Sequential,Pack=1)" => Some(Annotation::Packed),
        _ => None,
    }
}

/// Render a Tier-1 [`Annotation`] as the C# attribute body it maps to (without the leading `[`).
///
/// Returns `None` for concepts C# has no native form for (e.g. [`Annotation::Aligned`]).
pub fn annotation_to_csharp_attribute(annotation: &Annotation) -> Option<String> {
    match annotation {
        Annotation::ReprC => Some("StructLayout(LayoutKind.Sequential)".to_string()),
        Annotation::Packed => Some("StructLayout(LayoutKind.Sequential, Pack = 1)".to_string()),
        Annotation::Aligned(_) => None,
    }
}

/// Strip whitespace so spacing variants of the same attribute compare equal.
fn normalize(attribute: &str) -> String {
    attribute.chars().filter(|c| !c.is_whitespace()).collect()
}
