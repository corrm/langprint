use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult},
    ir::LanguageField,
};

use super::CppVisibility;

// TODO: Instead of `is_static` and `is_const` modifiers to CppField in langprint

/// Represents a field in a C++ struct.
#[derive(Debug, Clone)]
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
        let result_log = ConversionLog::new();

        let language_field = LanguageField {
            name: self.name,
            field_type: self.field_type,
            visibility: self.visibility.to_ir(None).value,
            array_size: self.array_size,
            bit_field_size: self.bit_field_size,
            initialization_value: self.initialization_value,
            inline_comment: self.inline_comment,
            docs: self.docs,
        };

        ConversionResult::with_log(language_field, result_log)
    }

    fn from_ir(
        input: Self::IrType,
        _options: Option<&Self::ConversionOptions>,
    ) -> ConversionResult<Self> {
        let mut result_log = ConversionLog::new();

        let visibility: ConversionResult<CppVisibility> =
            CppVisibility::from_ir(input.visibility, None);

        if visibility.log.has_warnings() {
            result_log.add_warnings(visibility.log.warnings);
        }

        let cpp_field = CppField {
            name: input.name,
            field_type: input.field_type,
            visibility: visibility.value,
            array_size: input.array_size,
            bit_field_size: input.bit_field_size,
            is_static: false, // Default value as IR doesn't have this concept
            is_const: false,  // Default value as IR doesn't have this concept
            is_inline: false, // Default value as IR doesn't have this concept
            initialization_value: input.initialization_value,
            inline_comment: input.inline_comment,
            docs: input.docs,
        };

        ConversionResult::with_log(cpp_field, result_log)
    }
}

/// Conversion options for C++ fields.
#[derive(Debug, Clone)]
pub struct CppFieldConversionOptions {}

impl Default for CppFieldConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CppFieldConversionOptions {
    pub const DEFAULT: Self = Self {};
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
