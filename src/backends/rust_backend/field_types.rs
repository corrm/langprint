use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    convert::{ConversionConfig, IdentifierKind, map_type, rename_identifier},
    ir::LanguageField,
    type_map::TargetLanguage,
};

use super::RustVisibility;

/// Represents a field of a Rust struct.
#[derive(Debug, Clone, PartialEq)]
pub struct RustField {
    /// The name of the field.
    pub name: String,
    /// The type of the field.
    pub field_type: String,
    /// The visibility of the field.
    pub visibility: RustVisibility,
    /// Attributes applied to the field (e.g. `#[serde(default)]`), without the leading `#[`.
    pub attributes: Vec<String>,
    /// Optional documentation for the field.
    pub docs: Option<Vec<String>>,
}

impl BackendItem for RustField {
    type IrType = LanguageField;
    type ConversionOptions = RustFieldConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut log = ConversionLog::new();

        for attribute in &self.attributes {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("attribute `#[{}]` on field `{}`", attribute, self.name),
                resolution: "Rust attributes are dropped from the language-agnostic IR".to_string(),
            });
        }

        let visibility = self.visibility.to_ir(None);
        log.add_warnings(visibility.log.warnings);

        ConversionResult::with_log(
            LanguageField {
                name: self.name,
                field_type: self.field_type,
                visibility: visibility.value,
                is_static: false,
                is_const: false,
                docs: self.docs,
            },
            log,
        )
    }

    fn from_ir(input: Self::IrType, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();
        let config = options.map(|options| options.config.clone()).unwrap_or_default();

        if input.is_static {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("static field `{}`", input.name),
                resolution: "Rust struct fields cannot be static; the modifier was dropped".to_string(),
            });
        }
        if input.is_const {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("const field `{}`", input.name),
                resolution: "Rust struct fields cannot be const; the modifier was dropped".to_string(),
            });
        }

        let name = rename_identifier(&config, &input.name, TargetLanguage::Rust, IdentifierKind::Field);
        log.add_warnings(name.log.warnings);

        let field_type = map_type(&config, &input.field_type, TargetLanguage::Rust);
        log.add_warnings(field_type.log.warnings);

        let visibility = RustVisibility::from_ir(input.visibility, None);
        log.add_warnings(visibility.log.warnings);

        ConversionResult::with_log(
            RustField {
                name: name.value,
                field_type: field_type.value,
                visibility: visibility.value,
                attributes: Vec::new(),
                docs: input.docs,
            },
            log,
        )
    }
}

/// Conversion options for Rust fields.
#[derive(Debug, Clone, Default)]
pub struct RustFieldConversionOptions {
    /// Cross-language conversion configuration (type mapping + renaming).
    pub config: ConversionConfig,
}

/// Render options for Rust fields.
#[derive(Debug, Clone)]
pub struct RustFieldRenderOptions {
    /// Whether to render documentation comments.
    pub render_docs: bool,
    /// Whether to render attributes.
    pub render_attributes: bool,
}

impl Default for RustFieldRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl RustFieldRenderOptions {
    pub const DEFAULT: Self = Self {
        render_docs: true,
        render_attributes: true,
    };
}
