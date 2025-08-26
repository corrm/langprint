use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult},
    ir::{EnumVariant, EnumVariantValue, LanguageEnum, Visibility},
};

use super::CppVisibility;

/// Represents a variant in a C++ enum.
#[derive(Debug, Clone)]
pub struct CppEnumVariant {
    /// The name of the variant.
    pub name: String,
    /// The value of the variant.
    pub value: Option<String>,
    /// Documentation for the variant.
    pub docs: Option<Vec<String>>,
}

impl BackendItem for CppEnumVariant {
    type IrType = EnumVariant;
    type ConversionOptions = CppEnumVariantConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        if let Some(value) = self.value {
            return ConversionResult::new(EnumVariant {
                name: self.name,
                value: EnumVariantValue::Value(value),
                docs: self.docs,
            });
        }

        ConversionResult::new(EnumVariant {
            name: self.name,
            value: EnumVariantValue::NoValue,
            docs: self.docs,
        })
    }

    fn from_ir(input: Self::IrType, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        ConversionResult::new(CppEnumVariant {
            name: input.name,
            value: match input.value {
                EnumVariantValue::NoValue => None,
                EnumVariantValue::Value(value) => Some(value),
                _ => None,
            },
            docs: input.docs,
        })
    }
}

/// Render options for C++ enum variants.
#[derive(Debug, Clone)]
pub struct CppEnumVariantRenderOptions {
    /// Whether to align the value of the variant with the variant name.
    pub align_value: bool,
    /// Whether to render documentation comments.
    pub render_docs: bool,
}

impl Default for CppEnumVariantRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CppEnumVariantRenderOptions {
    pub const DEFAULT: Self = Self {
        align_value: false,
        render_docs: true,
    };
}

/// Conversion options for C++ enum variants.
#[derive(Debug, Clone)]
pub struct CppEnumVariantConversionOptions {}

impl Default for CppEnumVariantConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CppEnumVariantConversionOptions {
    pub const DEFAULT: Self = Self {};
}

/// Represents a C++ enum definition.
#[derive(Debug, Clone)]
pub struct CppEnum {
    /// The name of the enum.
    pub name: String,
    /// The variants of the enum.
    pub variants: Vec<CppEnumVariant>,
    /// Whether this is a enum class.
    pub is_enum_class: bool,
    /// The underlying type of the enum (e.g., `int32_t`).
    pub underlying_type: Option<String>,
    /// Documentation for the enum.
    pub docs: Option<Vec<String>>,
}

impl BackendItem for CppEnum {
    type IrType = LanguageEnum;
    type ConversionOptions = CppEnumConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        ConversionResult::new(LanguageEnum {
            name: self.name,
            visibility: Visibility::Default,
            variants: self
                .variants
                .into_iter()
                .map(|variant| variant.to_ir(None).value)
                .collect(),
            underlying_type: self.underlying_type,
            docs: self.docs,
        })
    }

    fn from_ir(input: Self::IrType, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let options: &CppEnumConversionOptions = options.unwrap_or(&CppEnumConversionOptions::DEFAULT);
        let mut result_log = ConversionLog::new();

        let visibility: ConversionResult<CppVisibility> = CppVisibility::from_ir(input.visibility, None);
        if visibility.log.has_warnings() {
            result_log.add_warnings(visibility.log.warnings);
        }

        ConversionResult::with_log(
            CppEnum {
                name: input.name,
                variants: input
                    .variants
                    .into_iter()
                    .map(|variant: EnumVariant| CppEnumVariant::from_ir(variant, None).value)
                    .collect(),
                underlying_type: input.underlying_type,
                is_enum_class: options.is_enum_class,
                docs: input.docs,
            },
            result_log,
        )
    }
}

/// Conversion options for C++ enums.
#[derive(Debug, Clone)]
pub struct CppEnumConversionOptions {
    /// Whether to convert the enum to a C++ enum class.
    pub is_enum_class: bool,
}

impl Default for CppEnumConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CppEnumConversionOptions {
    pub const DEFAULT: Self = Self { is_enum_class: false };
}

/// Render options for C++ enums.
#[derive(Debug, Clone)]
pub struct CppEnumRenderOptions {
    /// Whether to render documentation comments.
    pub render_docs: bool,
}

impl Default for CppEnumRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CppEnumRenderOptions {
    pub const DEFAULT: Self = Self { render_docs: true };
}
