use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    convert::{ConversionConfig, map_type},
    ir::{EnumVariant, EnumVariantValue, LanguageEnum, Visibility},
    type_map::TargetLanguage,
};

use super::CppVisibility;

/// Represents a variant in a C++ enum.
#[derive(Debug, Clone, PartialEq)]
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
        let mut log = ConversionLog::new();

        let value = match input.value {
            EnumVariantValue::NoValue => None,
            EnumVariantValue::Value(value) => Some(value),
            EnumVariantValue::Tuple(_) | EnumVariantValue::Struct(_) => {
                log.add_warning(ConversionWarning::UnsupportedFeature {
                    feature: format!("data-carrying payload on enum variant `{}`", input.name),
                    resolution: "C++ enums hold no per-variant data; payload dropped".to_string(),
                });
                None
            }
        };

        ConversionResult::with_log(
            CppEnumVariant {
                name: input.name,
                value,
                docs: input.docs,
            },
            log,
        )
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
#[derive(Debug, Clone, PartialEq)]
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

    fn to_ir(self, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut result_log = ConversionLog::new();

        if !self.is_enum_class {
            result_log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("unscoped C++ enum `{}`", self.name),
                resolution: "represented as a scoped enum in the language-agnostic IR".to_string(),
            });
        }

        let mut language_enum = LanguageEnum {
            name: self.name,
            visibility: Visibility::Default,
            variants: self
                .variants
                .into_iter()
                .map(|variant| variant.to_ir(None).value)
                .collect(),
            underlying_type: self.underlying_type,
            docs: self.docs,
            annotations: Vec::new(),
            raw_attributes: Vec::new(),
        };
        if let Some(hooks) = options.and_then(|options| options.config.hooks.as_ref()) {
            hooks.after_to_ir_enum(&mut language_enum);
        }

        ConversionResult::with_log(language_enum, result_log)
    }

    fn from_ir(mut input: Self::IrType, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let default_options = CppEnumConversionOptions::default();
        let options: &CppEnumConversionOptions = options.unwrap_or(&default_options);
        let mut result_log = ConversionLog::new();
        if let Some(hooks) = &options.config.hooks {
            hooks.before_from_ir_enum(&mut input);
        }

        let visibility: ConversionResult<CppVisibility> = CppVisibility::from_ir(input.visibility, None);
        if visibility.log.has_warnings() {
            result_log.add_warnings(visibility.log.warnings);
        }

        let underlying_type = match input.underlying_type {
            Some(underlying_type) => {
                let mapped = map_type(&options.config, &underlying_type, TargetLanguage::Cpp);
                result_log.add_warnings(mapped.log.warnings);
                Some(mapped.value)
            }
            None => None,
        };

        let mut variants = Vec::with_capacity(input.variants.len());
        for variant in input.variants {
            let converted: ConversionResult<CppEnumVariant> = CppEnumVariant::from_ir(variant, None);
            if converted.log.has_warnings() {
                result_log.add_warnings(converted.log.warnings);
            }
            variants.push(converted.value);
        }

        for raw in &input.raw_attributes {
            if raw.source != TargetLanguage::Cpp {
                result_log.add_warning(ConversionWarning::UnsupportedFeature {
                    feature: format!("opaque {:?} attribute `{}`", raw.source, raw.text),
                    resolution: "cannot translate to C++; dropped".to_string(),
                });
            }
        }

        ConversionResult::with_log(
            CppEnum {
                name: input.name,
                variants,
                underlying_type,
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
    /// Cross-language conversion configuration (type mapping + renaming).
    pub config: ConversionConfig,
}

impl Default for CppEnumConversionOptions {
    fn default() -> Self {
        Self {
            is_enum_class: true,
            config: ConversionConfig::default(),
        }
    }
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
