//! Mapping between Rust native attribute strings and the neutral [`Annotation`] vocabulary.

use crate::ir::Annotation;

/// Recognize a Rust attribute body (without the leading `#[`) as a Tier-1 [`Annotation`].
///
/// Returns `None` when the attribute has no curated mapping; the caller carries it as a
/// [`crate::ir::RawAttribute`] instead.
pub fn rust_attribute_to_annotation(attribute: &str) -> Option<Annotation> {
    match attribute.trim() {
        "repr(C)" => Some(Annotation::ReprC),
        "repr(packed)" => Some(Annotation::Packed),
        other => parse_repr_align(other).map(Annotation::Aligned),
    }
}

/// Parse `repr(align(N))` into `N`.
fn parse_repr_align(attribute: &str) -> Option<u32> {
    let inner = attribute.strip_prefix("repr(align(")?.strip_suffix("))")?;
    inner.trim().parse().ok()
}
