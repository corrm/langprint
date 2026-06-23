use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    ir::LanguageField,
};

use super::RustVisibility;

/// Represents a field of a Rust struct.
#[derive(Debug, Clone)]
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

    fn from_ir(input: Self::IrType, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();

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

        let visibility = RustVisibility::from_ir(input.visibility, None);
        log.add_warnings(visibility.log.warnings);

        ConversionResult::with_log(
            RustField {
                name: input.name,
                field_type: input.field_type,
                visibility: visibility.value,
                attributes: Vec::new(),
                docs: input.docs,
            },
            log,
        )
    }
}

/// Conversion options for Rust fields.
#[derive(Debug, Clone)]
pub struct RustFieldConversionOptions {}

impl Default for RustFieldConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl RustFieldConversionOptions {
    pub const DEFAULT: Self = Self {};
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
