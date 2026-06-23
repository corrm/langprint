use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    ir::LanguageField,
};

use super::CSharpVisibility;

/// Represents a C# field.
#[derive(Debug, Clone)]
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

    fn from_ir(input: Self::IrType, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let visibility = CSharpVisibility::from_ir(input.visibility, None);
        let field = CSharpField {
            name: input.name,
            field_type: input.field_type,
            visibility: visibility.value,
            is_static: input.is_static,
            is_const: input.is_const,
            is_readonly: false,
            initializer: None,
            attributes: Vec::new(),
            docs: input.docs,
        };
        ConversionResult::with_log(field, visibility.log)
    }
}

/// Conversion options for C# fields.
#[derive(Debug, Clone)]
pub struct CSharpFieldConversionOptions {}

impl Default for CSharpFieldConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CSharpFieldConversionOptions {
    pub const DEFAULT: Self = Self {};
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
