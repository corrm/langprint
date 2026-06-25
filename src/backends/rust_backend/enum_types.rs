use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    convert::{ConversionConfig, map_type},
    ir::{EnumVariant, EnumVariantValue, LanguageEnum},
    type_map::TargetLanguage,
};

use super::RustVisibility;

/// Whether a `#[repr(...)]` token is an integral representation that maps to an enum underlying type.
fn is_integral_repr(repr: &str) -> bool {
    matches!(
        repr,
        "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64" | "i128" | "u128" | "isize" | "usize"
    )
}

/// The payload of a Rust enum variant.
#[derive(Debug, Clone, PartialEq)]
pub enum RustEnumVariantValue {
    /// A unit variant (`Foo`).
    Unit,
    /// A unit variant with an explicit discriminant (`Foo = 1`).
    Discriminant(String),
    /// A tuple variant (`Foo(A, B)`).
    Tuple(Vec<String>),
    /// A struct variant (`Foo { a: A, b: B }`).
    Struct(Vec<(String, String)>),
}

/// A variant of a Rust enum.
#[derive(Debug, Clone, PartialEq)]
pub struct RustEnumVariant {
    /// The name of the variant.
    pub name: String,
    /// The payload of the variant.
    pub value: RustEnumVariantValue,
    /// Optional documentation for the variant.
    pub docs: Option<Vec<String>>,
}

/// Represents a Rust enum.
#[derive(Debug, Clone, PartialEq)]
pub struct RustEnum {
    /// The name of the enum.
    pub name: String,
    /// The visibility of the enum.
    pub visibility: RustVisibility,
    /// The variants of the enum.
    pub variants: Vec<RustEnumVariant>,
    /// The `#[repr(...)]` representation, if any (e.g. `u8`).
    pub repr: Option<String>,
    /// Derives applied to the enum (e.g. `Debug`, `Clone`).
    pub derives: Vec<String>,
    /// Optional documentation for the enum.
    pub docs: Option<Vec<String>>,
}

impl BackendItem for RustEnum {
    type IrType = LanguageEnum;
    type ConversionOptions = RustEnumConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut log = ConversionLog::new();

        if !self.derives.is_empty() {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("derives on enum `{}`", self.name),
                resolution: "Rust derives are dropped from the language-agnostic IR".to_string(),
            });
        }

        let underlying_type = match self.repr {
            Some(repr) if is_integral_repr(&repr) => Some(repr),
            Some(repr) => {
                log.add_warning(ConversionWarning::UnsupportedFeature {
                    feature: format!("#[repr({repr})] on enum `{}`", self.name),
                    resolution: "only integral reprs map to an enum underlying type; dropped".to_string(),
                });
                None
            }
            None => None,
        };

        let visibility = self.visibility.to_ir(None);
        log.add_warnings(visibility.log.warnings);

        let variants = self
            .variants
            .into_iter()
            .map(|variant| EnumVariant {
                name: variant.name,
                value: match variant.value {
                    RustEnumVariantValue::Unit => EnumVariantValue::NoValue,
                    RustEnumVariantValue::Discriminant(value) => EnumVariantValue::Value(value),
                    RustEnumVariantValue::Tuple(types) => EnumVariantValue::Tuple(types),
                    RustEnumVariantValue::Struct(fields) => EnumVariantValue::Struct(fields),
                },
                docs: variant.docs,
            })
            .collect();

        ConversionResult::with_log(
            LanguageEnum {
                name: self.name,
                visibility: visibility.value,
                variants,
                underlying_type,
                docs: self.docs,
            },
            log,
        )
    }

    fn from_ir(input: Self::IrType, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();
        let config = options.map(|options| options.config.clone()).unwrap_or_default();

        let visibility = RustVisibility::from_ir(input.visibility, None);
        log.add_warnings(visibility.log.warnings);

        let repr = match input.underlying_type {
            Some(underlying_type) => {
                let mapped = map_type(&config, &underlying_type, TargetLanguage::Rust);
                log.add_warnings(mapped.log.warnings);
                Some(mapped.value)
            }
            None => None,
        };

        let variants = input
            .variants
            .into_iter()
            .map(|variant| RustEnumVariant {
                name: variant.name,
                value: match variant.value {
                    EnumVariantValue::NoValue => RustEnumVariantValue::Unit,
                    EnumVariantValue::Value(value) => RustEnumVariantValue::Discriminant(value),
                    EnumVariantValue::Tuple(types) => RustEnumVariantValue::Tuple(types),
                    EnumVariantValue::Struct(fields) => RustEnumVariantValue::Struct(fields),
                },
                docs: variant.docs,
            })
            .collect();

        ConversionResult::with_log(
            RustEnum {
                name: input.name,
                visibility: visibility.value,
                variants,
                repr,
                derives: Vec::new(),
                docs: input.docs,
            },
            log,
        )
    }
}

/// Conversion options for Rust enums.
#[derive(Debug, Clone, Default)]
pub struct RustEnumConversionOptions {
    /// Cross-language conversion configuration (type mapping + renaming).
    pub config: ConversionConfig,
}

/// Render options for Rust enum variants.
#[derive(Debug, Clone)]
pub struct RustEnumVariantRenderOptions {
    /// Whether to render documentation comments.
    pub render_docs: bool,
}

impl Default for RustEnumVariantRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl RustEnumVariantRenderOptions {
    pub const DEFAULT: Self = Self { render_docs: true };
}

/// Render options for Rust enums.
#[derive(Debug, Clone)]
pub struct RustEnumRenderOptions {
    /// Whether to render documentation comments.
    pub render_docs: bool,
    /// Whether to render derives and the `#[repr(...)]` attribute.
    pub render_attributes: bool,
}

impl Default for RustEnumRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl RustEnumRenderOptions {
    pub const DEFAULT: Self = Self {
        render_docs: true,
        render_attributes: true,
    };
}
