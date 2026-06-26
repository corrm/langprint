use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    convert::{ConversionConfig, IdentifierKind, map_type, rename_identifier},
    ir::{EnumVariant, EnumVariantValue, LanguageEnum, RawAttribute},
    type_map::TargetLanguage,
};

use super::RustVisibility;
use super::attributes::rust_attribute_to_annotation;

/// Whether a `#[repr(...)]` token is an integral representation that maps to an enum underlying type.
fn is_integral_repr(repr: &str) -> bool {
    matches!(
        repr,
        "i8" | "u8"
            | "i16"
            | "u16"
            | "i32"
            | "u32"
            | "i64"
            | "u64"
            | "i128"
            | "u128"
            | "isize"
            | "usize"
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

    fn to_ir(self, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut log = ConversionLog::new();

        let mut annotations = Vec::new();
        let mut raw_attributes = Vec::new();
        for derive in &self.derives {
            raw_attributes.push(RawAttribute {
                source: TargetLanguage::Rust,
                text: format!("derive({derive})"),
            });
        }

        let underlying_type = match self.repr {
            Some(repr) if is_integral_repr(&repr) => Some(repr),
            Some(repr) => {
                match rust_attribute_to_annotation(&format!("repr({repr})")) {
                    Some(annotation) => annotations.push(annotation),
                    None => raw_attributes.push(RawAttribute {
                        source: TargetLanguage::Rust,
                        text: format!("repr({repr})"),
                    }),
                }
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

        let mut ir = LanguageEnum {
            name: self.name,
            visibility: visibility.value,
            variants,
            underlying_type,
            docs: self.docs,
            annotations,
            raw_attributes,
        };
        if let Some(hooks) = options.and_then(|options| options.config.hooks.as_ref()) {
            hooks.after_to_ir_enum(&mut ir);
        }

        ConversionResult::with_log(ir, log)
    }

    fn from_ir(
        mut input: Self::IrType,
        options: Option<&Self::ConversionOptions>,
    ) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();
        let config = options
            .map(|options| options.config.clone())
            .unwrap_or_default();
        if let Some(hooks) = &config.hooks {
            hooks.before_from_ir_enum(&mut input);
        }

        let name = rename_identifier(
            &config,
            &input.name,
            TargetLanguage::Rust,
            IdentifierKind::Type,
        );
        log.add_warnings(name.log.warnings);

        let visibility = RustVisibility::from_ir(input.visibility, None);
        log.add_warnings(visibility.log.warnings);

        let mut derives = Vec::new();
        for raw in &input.raw_attributes {
            if raw.source != TargetLanguage::Rust {
                log.add_warning(ConversionWarning::UnsupportedFeature {
                    feature: format!("opaque {:?} attribute `{}`", raw.source, raw.text),
                    resolution: "cannot translate to Rust; dropped".to_string(),
                });
                continue;
            }
            if let Some(derive) = raw
                .text
                .strip_prefix("derive(")
                .and_then(|rest| rest.strip_suffix(")"))
            {
                derives.push(derive.to_string());
            }
        }

        let repr = match input.underlying_type {
            Some(underlying_type) => {
                let mapped = map_type(&config, &underlying_type, TargetLanguage::Rust);
                log.add_warnings(mapped.log.warnings);
                Some(mapped.value)
            }
            None => None,
        };

        let mut variants = Vec::with_capacity(input.variants.len());
        for variant in input.variants {
            let variant_name = rename_identifier(
                &config,
                &variant.name,
                TargetLanguage::Rust,
                IdentifierKind::EnumMember,
            );
            log.add_warnings(variant_name.log.warnings);
            variants.push(RustEnumVariant {
                name: variant_name.value,
                value: match variant.value {
                    EnumVariantValue::NoValue => RustEnumVariantValue::Unit,
                    EnumVariantValue::Value(value) => RustEnumVariantValue::Discriminant(value),
                    EnumVariantValue::Tuple(types) => RustEnumVariantValue::Tuple(types),
                    EnumVariantValue::Struct(fields) => RustEnumVariantValue::Struct(fields),
                },
                docs: variant.docs,
            });
        }

        ConversionResult::with_log(
            RustEnum {
                name: name.value,
                visibility: visibility.value,
                variants,
                repr,
                derives,
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
    /// Render options for the enum's variants.
    pub variant: RustEnumVariantRenderOptions,
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
        variant: RustEnumVariantRenderOptions::DEFAULT,
    };
}
