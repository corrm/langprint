//! Shared cross-language conversion configuration and the leaf helpers every backend's `from_ir`
//! uses to re-spell types and apply idiomatic identifier renaming.

use std::sync::Arc;

use crate::conversion::{ConversionResult, ConversionWarning};
use crate::naming::{to_pascal_case, to_snake_case};
use crate::type_map::{TargetLanguage, TypeMap};

impl TargetLanguage {
    /// The human-readable language name used in conversion warnings.
    pub fn name(self) -> &'static str {
        match self {
            TargetLanguage::Cpp => "C++",
            TargetLanguage::Rust => "Rust",
            TargetLanguage::CSharp => "C#",
        }
    }
}

/// Configuration shared by every `from_ir` conversion: how primitive types are re-spelled and
/// whether identifiers are renamed to the target language's convention.
#[derive(Debug, Clone)]
pub struct ConversionConfig {
    /// The primitive type mapping applied at conversion boundaries.
    pub type_map: Arc<TypeMap>,
    /// Whether identifiers are renamed to the target language's naming convention.
    pub rename: bool,
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            type_map: Arc::new(TypeMap::builtin()),
            rename: true,
        }
    }
}

impl ConversionConfig {
    /// Create a configuration from a type map and a rename flag.
    pub fn new(type_map: TypeMap, rename: bool) -> Self {
        Self {
            type_map: Arc::new(type_map),
            rename,
        }
    }
}

/// The kind of identifier being renamed; selects the target convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentifierKind {
    /// A type name (struct/class/enum).
    Type,
    /// A function or method name.
    Function,
    /// A field name.
    Field,
    /// An enum member name.
    EnumMember,
}

/// The case style an identifier is rewritten into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaseStyle {
    Snake,
    Pascal,
}

/// The naming convention a language applies to an identifier kind, if any.
fn convention(language: TargetLanguage, kind: IdentifierKind) -> Option<CaseStyle> {
    match language {
        TargetLanguage::Cpp => None,
        TargetLanguage::Rust => match kind {
            IdentifierKind::Function | IdentifierKind::Field => Some(CaseStyle::Snake),
            IdentifierKind::Type | IdentifierKind::EnumMember => None,
        },
        TargetLanguage::CSharp => Some(CaseStyle::Pascal),
    }
}

/// Re-spell a type into the target language using the configured [`TypeMap`].
///
/// # Arguments
///
/// * `config` - The active conversion configuration.
/// * `spelling` - The source type spelling.
/// * `language` - The target language.
///
/// # Returns
///
/// The mapped spelling, or the original spelling plus an `UnsupportedFeature` warning when the
/// type is not a recognized primitive.
pub fn map_type(config: &ConversionConfig, spelling: &str, language: TargetLanguage) -> ConversionResult<String> {
    match config.type_map.map(spelling, language) {
        Some(mapped) => ConversionResult::new(mapped),
        None => ConversionResult::with_warning(
            spelling.to_string(),
            ConversionWarning::UnsupportedFeature {
                feature: format!("unmapped type `{spelling}`"),
                resolution: format!("no TypeMap entry for {}; emitted verbatim", language.name()),
            },
        ),
    }
}

/// Rename an identifier to the target language's convention when renaming is enabled.
///
/// # Arguments
///
/// * `config` - The active conversion configuration.
/// * `name` - The source identifier.
/// * `language` - The target language.
/// * `kind` - The kind of identifier, which selects the convention.
///
/// # Returns
///
/// The renamed identifier, or the original; a `NamingConventionChanged` warning is attached only
/// when the identifier actually changes.
pub fn rename_identifier(
    config: &ConversionConfig,
    name: &str,
    language: TargetLanguage,
    kind: IdentifierKind,
) -> ConversionResult<String> {
    if !config.rename {
        return ConversionResult::new(name.to_string());
    }

    let Some(style) = convention(language, kind) else {
        return ConversionResult::new(name.to_string());
    };

    let converted = match style {
        CaseStyle::Snake => to_snake_case(name),
        CaseStyle::Pascal => to_pascal_case(name),
    };

    if converted == name {
        return ConversionResult::new(converted);
    }

    ConversionResult::with_warning(
        converted.clone(),
        ConversionWarning::NamingConventionChanged {
            original: name.to_string(),
            converted,
        },
    )
}
