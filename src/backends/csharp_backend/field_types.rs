use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    convert::{ConversionConfig, IdentifierKind, map_type, rename_identifier},
    ir::LanguageField,
    type_map::TargetLanguage,
};

use super::CSharpVisibility;

/// Represents a C# field.
#[derive(Debug, Clone, PartialEq)]
pub struct CSharpField {
    /// The name of the field.
    pub name: String,
    /// The type of the field.
    pub field_type: String,
    /// The visibility of the field.
    pub visibility: CSharpVisibility,
    /// Whether the field is `static`.
    pub is_static: bool,
    /// Whether the field is `const`.
    pub is_const: bool,
    /// Whether the field is `readonly`.
    pub is_readonly: bool,
    /// Optional initializer expression (e.g. `0`).
    pub initializer: Option<String>,
    /// Attributes applied to the field (without the leading `[`, e.g. `NonSerialized`).
    pub attributes: Vec<String>,
    /// Documentation for the field.
    pub docs: Option<Vec<String>>,
}

impl BackendItem for CSharpField {
    type IrType = LanguageField;
    type ConversionOptions = CSharpFieldConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut log = ConversionLog::new();

        if self.is_readonly {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("`readonly` on field `{}`", self.name),
                resolution: "readonly modifier dropped from the language-agnostic IR".to_string(),
            });
        }
        if self.initializer.is_some() {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("initializer on field `{}`", self.name),
                resolution: "field initializer dropped from the language-agnostic IR".to_string(),
            });
        }
        for attribute in &self.attributes {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("attribute `[{}]` on field `{}`", attribute, self.name),
                resolution: "C# attributes dropped from the language-agnostic IR".to_string(),
            });
        }

        let visibility = self.visibility.to_ir(None);
        log.add_warnings(visibility.log.warnings);

        let field = LanguageField {
            name: self.name,
            field_type: self.field_type,
            visibility: visibility.value,
            is_static: self.is_static,
            is_const: self.is_const,
            docs: self.docs,
        };
        ConversionResult::with_log(field, log)
    }

    fn from_ir(input: Self::IrType, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();
        let config = options.map(|options| options.config.clone()).unwrap_or_default();

        let name = rename_identifier(&config, &input.name, TargetLanguage::CSharp, IdentifierKind::Field);
        log.add_warnings(name.log.warnings);

        let field_type = map_type(&config, &input.field_type, TargetLanguage::CSharp);
        log.add_warnings(field_type.log.warnings);

        let visibility = CSharpVisibility::from_ir(input.visibility, None);
        log.add_warnings(visibility.log.warnings);

        let field = CSharpField {
            name: name.value,
            field_type: field_type.value,
            visibility: visibility.value,
            is_static: input.is_static,
            is_const: input.is_const,
            is_readonly: false,
            initializer: None,
            attributes: Vec::new(),
            docs: input.docs,
        };
        ConversionResult::with_log(field, log)
    }
}

/// Conversion options for C# fields.
#[derive(Debug, Clone, Default)]
pub struct CSharpFieldConversionOptions {
    /// Cross-language conversion configuration (type mapping + renaming).
    pub config: ConversionConfig,
}

/// Render options for C# fields.
#[derive(Debug, Clone)]
pub struct CSharpFieldRenderOptions {
    /// Whether to render documentation comments.
    pub render_docs: bool,
    /// Whether to render attributes.
    pub render_attributes: bool,
}

impl Default for CSharpFieldRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CSharpFieldRenderOptions {
    pub const DEFAULT: Self = Self {
        render_docs: true,
        render_attributes: true,
    };
}
