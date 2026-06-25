//! Shared cross-language conversion configuration and the leaf helpers every backend's `from_ir`
//! uses to re-spell types and apply idiomatic identifier renaming.

use std::fmt;
use std::sync::Arc;

use crate::conversion::{ConversionResult, ConversionWarning};
use crate::ir::{LanguageEnum, LanguageFunction, LanguageStruct};
use crate::naming::{to_camel_case, to_pascal_case, to_snake_case};
use crate::type_map::{TargetLanguage, TypeMap};

impl TargetLanguage {
    /// The human-readable language name used in conversion warnings.
    pub fn name(self) -> &'static str {
        match self {
            TargetLanguage::Cpp => "C++",
            TargetLanguage::Rust => "Rust",
            TargetLanguage::CSharp => "C#",
            TargetLanguage::Python => "Python",
            TargetLanguage::Lua => "Lua",
            TargetLanguage::Js => "JS",
        }
    }
}

/// A custom type resolver consulted before the [`TypeMap`]: given a source spelling and the target
/// language, it returns `Some(spelling)` to override the mapping or `None` to defer to the map.
pub type TypeOverride = Arc<dyn Fn(&str, TargetLanguage) -> Option<String> + Send + Sync>;

/// Opt-in lifecycle hooks invoked on the cross-language IR path. Every method is a no-op by
/// default, so existing behavior is unchanged unless a hook is set. Hooks mutate the IR item in
/// place: `after_to_ir_*` fires once the IR value is built, `before_from_ir_*` fires before the IR
/// is lowered into a target backend type.
pub trait ConversionHooks: Send + Sync {
    /// Invoked after a struct/class has been raised into the IR.
    fn after_to_ir_struct(&self, _s: &mut LanguageStruct) {}
    /// Invoked after a function/method has been raised into the IR.
    fn after_to_ir_function(&self, _f: &mut LanguageFunction) {}
    /// Invoked after an enum has been raised into the IR.
    fn after_to_ir_enum(&self, _e: &mut LanguageEnum) {}
    /// Invoked before a struct/class is lowered out of the IR.
    fn before_from_ir_struct(&self, _s: &mut LanguageStruct) {}
    /// Invoked before a function/method is lowered out of the IR.
    fn before_from_ir_function(&self, _f: &mut LanguageFunction) {}
    /// Invoked before an enum is lowered out of the IR.
    fn before_from_ir_enum(&self, _e: &mut LanguageEnum) {}
}

/// Configuration shared by every `from_ir` conversion: how primitive types are re-spelled and
/// whether identifiers are renamed to the target language's convention.
#[derive(Clone)]
pub struct ConversionConfig {
    /// The primitive type mapping applied at conversion boundaries.
    pub type_map: Arc<TypeMap>,
    /// Whether identifiers are renamed to the target language's naming convention.
    pub rename: bool,
    /// A custom type resolver consulted before the [`TypeMap`].
    pub type_override: Option<TypeOverride>,
    /// Opt-in lifecycle hooks invoked on the cross-language IR path.
    pub hooks: Option<Arc<dyn ConversionHooks>>,
}

impl fmt::Debug for ConversionConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConversionConfig")
            .field("type_map", &self.type_map)
            .field("rename", &self.rename)
            .field("type_override", &self.type_override.as_ref().map(|_| "<fn>"))
            .field("hooks", &self.hooks.as_ref().map(|_| "<hooks>"))
            .finish()
    }
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            type_map: Arc::new(TypeMap::builtin()),
            rename: true,
            type_override: None,
            hooks: None,
        }
    }
}

impl ConversionConfig {
    /// Create a configuration from a type map and a rename flag.
    pub fn new(type_map: TypeMap, rename: bool) -> Self {
        Self {
            type_map: Arc::new(type_map),
            rename,
            type_override: None,
            hooks: None,
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
    /// A namespace / module name.
    Namespace,
}

/// The case style an identifier is rewritten into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaseStyle {
    Snake,
    Pascal,
    Camel,
}

/// The naming convention a language applies to an identifier kind, if any.
fn convention(language: TargetLanguage, kind: IdentifierKind) -> Option<CaseStyle> {
    match language {
        TargetLanguage::Cpp => None,
        TargetLanguage::Rust => match kind {
            IdentifierKind::Function | IdentifierKind::Field | IdentifierKind::Namespace => Some(CaseStyle::Snake),
            IdentifierKind::Type | IdentifierKind::EnumMember => None,
        },
        TargetLanguage::CSharp => Some(CaseStyle::Pascal),
        TargetLanguage::Python => match kind {
            IdentifierKind::Function | IdentifierKind::Field | IdentifierKind::Namespace => Some(CaseStyle::Snake),
            IdentifierKind::Type => Some(CaseStyle::Pascal),
            IdentifierKind::EnumMember => None,
        },
        TargetLanguage::Lua => match kind {
            IdentifierKind::Function | IdentifierKind::Field | IdentifierKind::Namespace => Some(CaseStyle::Snake),
            IdentifierKind::Type | IdentifierKind::EnumMember => None,
        },
        TargetLanguage::Js => match kind {
            IdentifierKind::Function | IdentifierKind::Field => Some(CaseStyle::Camel),
            IdentifierKind::Type => Some(CaseStyle::Pascal),
            IdentifierKind::Namespace | IdentifierKind::EnumMember => None,
        },
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
    if let Some(resolver) = &config.type_override
        && let Some(mapped) = resolver(spelling, language)
    {
        return ConversionResult::new(mapped);
    }

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
        CaseStyle::Camel => to_camel_case(name),
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
