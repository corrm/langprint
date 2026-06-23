use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    ir::{EnumVariant, EnumVariantValue, LanguageEnum},
};

use super::RustVisibility;

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
#[derive(Debug, Clone)]
pub struct RustEnumVariant {
    /// The name of the variant.
    pub name: String,
    /// The payload of the variant.
    pub value: RustEnumVariantValue,
    /// Optional documentation for the variant.
    pub docs: Option<Vec<String>>,
}

/// Represents a Rust enum.
#[derive(Debug, Clone)]
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
                underlying_type: self.repr,
                docs: self.docs,
            },
            log,
        )
    }

    fn from_ir(input: Self::IrType, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();

        let visibility = RustVisibility::from_ir(input.visibility, None);
        log.add_warnings(visibility.log.warnings);

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
                repr: input.underlying_type,
                derives: Vec::new(),
                docs: input.docs,
            },
            log,
        )
    }
}

/// Conversion options for Rust enums.
#[derive(Debug, Clone)]
pub struct RustEnumConversionOptions {}

impl Default for RustEnumConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl RustEnumConversionOptions {
    pub const DEFAULT: Self = Self {};
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
