use super::BackendFeature;

/// Trait for providing metadata about a language backend.
pub trait BackendMetadata {
    /// Get the name of the language.
    fn language_name(&self) -> &'static str;

    /// Get the features supported by this language.
    fn supported_features(&self) -> &'static [BackendFeature];
}
