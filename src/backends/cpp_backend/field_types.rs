use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    convert::{ConversionConfig, IdentifierKind, map_type, rename_identifier},
    ir::{Annotation, LanguageField},
    type_map::TargetLanguage,
};

use super::CppVisibility;

/// Represents a field in a C++ struct.
#[derive(Debug, Clone, PartialEq)]
pub struct CppField {
    /// The name of the field.
    pub name: String,
    /// The type of the field.
    pub field_type: String,
    /// The visibility of the field.
    pub visibility: CppVisibility,
    /// The size of the array if the field is an array.
    pub array_size: Option<String>,
    /// Bit field size (can be a number or a macro/define name).
    pub bit_field_size: Option<String>,
    /// Over-alignment for this field (`alignas(N)`); `None` = natural alignment.
    pub alignment: Option<u32>,
    /// Whether the field is static.
    pub is_static: bool,
    /// Whether the field is const.
    pub is_const: bool,
    /// Whether the field is inline.
    pub is_inline: bool,
    /// Optional initialization value for the field.
    pub initialization_value: Option<String>,
    /// Inline comment for the field.
    pub inline_comment: Option<String>,
    /// Documentation for the field.
    pub docs: Option<Vec<String>>,
}

impl BackendItem for CppField {
    type IrType = LanguageField;
    type ConversionOptions = CppFieldConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut result_log = ConversionLog::new();

        let visibility = self.visibility.to_ir(None);
        if visibility.log.has_warnings() {
            result_log.add_warnings(visibility.log.warnings);
        }

        if self.array_size.is_some() {
            result_log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("C array dimension on field `{}`", self.name),
                resolution: "array size dropped; encode it in the field type if needed".to_string(),
            });
        }
        if self.bit_field_size.is_some() {
            result_log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("C bit-field on field `{}`", self.name),
                resolution: "bit-field width dropped from the language-agnostic IR".to_string(),
            });
        }
        let mut annotations = Vec::new();
        if let Some(alignment) = self.alignment {
            annotations.push(Annotation::Aligned(alignment));
        }
        if self.is_inline {
            result_log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("`inline` specifier on field `{}`", self.name),
                resolution: "inline specifier dropped from the language-agnostic IR".to_string(),
            });
        }
        if self.initialization_value.is_some() {
            result_log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("initializer on field `{}`", self.name),
                resolution: "field initializer dropped from the language-agnostic IR".to_string(),
            });
        }
        if self.inline_comment.is_some() {
            result_log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("inline comment on field `{}`", self.name),
                resolution: "inline comment dropped from the language-agnostic IR".to_string(),
            });
        }

        let language_field = LanguageField {
            name: self.name,
            field_type: self.field_type,
            visibility: visibility.value,
            is_static: self.is_static,
            is_const: self.is_const,
            docs: self.docs,
            annotations,
            raw_attributes: Vec::new(),
        };

        ConversionResult::with_log(language_field, result_log)
    }

    fn from_ir(
        input: Self::IrType,
        options: Option<&Self::ConversionOptions>,
    ) -> ConversionResult<Self> {
        let mut result_log = ConversionLog::new();
        let config = options
            .map(|options| options.config.clone())
            .unwrap_or_default();

        let visibility: ConversionResult<CppVisibility> =
            CppVisibility::from_ir(input.visibility, None);

        if visibility.log.has_warnings() {
            result_log.add_warnings(visibility.log.warnings);
        }

        let field_type = map_type(&config, &input.field_type, TargetLanguage::Cpp);
        result_log.add_warnings(field_type.log.warnings);

        let mut alignment = None;
        for annotation in &input.annotations {
            if let Annotation::Aligned(n) = annotation {
                alignment = Some(*n);
            }
        }
        for raw in &input.raw_attributes {
            if raw.source != TargetLanguage::Cpp {
                result_log.add_warning(ConversionWarning::UnsupportedFeature {
                    feature: format!("opaque {:?} attribute `{}`", raw.source, raw.text),
                    resolution: "cannot translate to C++; dropped".to_string(),
                });
            }
        }

        let name = rename_identifier(
            &config,
            &input.name,
            TargetLanguage::Cpp,
            IdentifierKind::Field,
        );
        result_log.add_warnings(name.log.warnings);

        let cpp_field = CppField {
            name: name.value,
            field_type: field_type.value,
            visibility: visibility.value,
            array_size: None,
            bit_field_size: None,
            alignment,
            is_static: input.is_static,
            is_const: input.is_const,
            is_inline: false,
            initialization_value: None,
            inline_comment: None,
            docs: input.docs,
        };

        ConversionResult::with_log(cpp_field, result_log)
    }
}

/// Conversion options for C++ fields.
#[derive(Debug, Clone, Default)]
pub struct CppFieldConversionOptions {
    /// Cross-language conversion configuration (type mapping + renaming).
    pub config: ConversionConfig,
}

/// Render options for C++ fields.
#[derive(Debug, Clone)]
pub struct CppFieldRenderOptions {
    /// Whether to include field initializers in the declaration.
    pub render_initializers: bool,
    /// Whether to render static fields with their storage class specifier.
    pub render_static_specifier: bool,
    /// Whether to render const fields with their const qualifier.
    pub render_const_qualifier: bool,
    /// Whether to render documentation comments.
    pub render_docs: bool,
}

impl Default for CppFieldRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CppFieldRenderOptions {
    pub const DEFAULT: Self = Self {
        render_initializers: true,
        render_static_specifier: true,
        render_const_qualifier: true,
        render_docs: true,
    };
}
