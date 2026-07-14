use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    convert::{ConversionConfig, IdentifierKind, map_type, rename_identifier},
    ir::{EnumVariant, EnumVariantValue, LanguageEnum, RawAttribute},
    type_map::TargetLanguage,
};

use super::CSharpVisibility;
use super::attributes::csharp_attribute_to_annotation;

/// Represents a single member of a C# enum.
#[derive(Debug, Clone, PartialEq)]
pub struct CSharpEnumMember {
    /// The name of the member.
    pub name: String,
    /// The explicit value of the member, if any.
    pub value: Option<String>,
    /// Documentation for the member.
    pub docs: Option<Vec<String>>,
}

/// Represents a C# enum.
#[derive(Debug, Clone, PartialEq)]
pub struct CSharpEnum {
    /// The name of the enum.
    pub name: String,
    /// The visibility of the enum.
    pub visibility: CSharpVisibility,
    /// The underlying integral type (e.g. `byte`), if specified.
    pub underlying_type: Option<String>,
    /// The members of the enum.
    pub members: Vec<CSharpEnumMember>,
    /// Whether the enum carries the `[Flags]` attribute.
    pub is_flags: bool,
    /// Additional attributes applied to the enum (without the leading `[`).
    pub attributes: Vec<String>,
    /// Documentation for the enum.
    pub docs: Option<Vec<String>>,
}

impl BackendItem for CSharpEnum {
    type IrType = LanguageEnum;
    type ConversionOptions = CSharpEnumConversionOptions;

    fn to_ir(self, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut log = ConversionLog::new();

        let mut annotations = Vec::new();
        let mut raw_attributes = Vec::new();
        if self.is_flags {
            raw_attributes.push(RawAttribute {
                source: TargetLanguage::CSharp,
                text: "Flags".to_string(),
            });
        }
        for attribute in &self.attributes {
            match csharp_attribute_to_annotation(attribute) {
                Some(annotation) => annotations.push(annotation),
                None => raw_attributes.push(RawAttribute {
                    source: TargetLanguage::CSharp,
                    text: attribute.clone(),
                }),
            }
        }

        let visibility = self.visibility.to_ir(None);
        log.add_warnings(visibility.log.warnings);

        let variants = self
            .members
            .into_iter()
            .map(|member| EnumVariant {
                name: member.name,
                value: match member.value {
                    Some(value) => EnumVariantValue::Value(value),
                    None => EnumVariantValue::NoValue,
                },
                docs: member.docs,
                raw_attributes: Vec::new(),
            })
            .collect();

        let mut language_enum = LanguageEnum {
            name: self.name,
            visibility: visibility.value,
            variants,
            underlying_type: self.underlying_type,
            docs: self.docs,
            annotations,
            raw_attributes,
        };
        if let Some(hooks) = options.and_then(|options| options.config.hooks.as_ref()) {
            hooks.after_to_ir_enum(&mut language_enum);
        }
        ConversionResult::with_log(language_enum, log)
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
            TargetLanguage::CSharp,
            IdentifierKind::Type,
        );
        log.add_warnings(name.log.warnings);

        let visibility = CSharpVisibility::from_ir(input.visibility, None);
        log.add_warnings(visibility.log.warnings);

        let mut is_flags = false;
        let mut attributes = Vec::new();
        for raw in &input.raw_attributes {
            if raw.source != TargetLanguage::CSharp {
                log.add_warning(ConversionWarning::UnsupportedFeature {
                    feature: format!("opaque {:?} attribute `{}`", raw.source, raw.text),
                    resolution: "cannot translate to C#; dropped".to_string(),
                });
                continue;
            }
            if raw.text == "Flags" {
                is_flags = true;
            } else {
                attributes.push(raw.text.clone());
            }
        }

        let underlying_type = match input.underlying_type {
            Some(underlying_type) => {
                let mapped = map_type(&config, &underlying_type, TargetLanguage::CSharp);
                log.add_warnings(mapped.log.warnings);
                Some(mapped.value)
            }
            None => None,
        };

        let mut members = Vec::with_capacity(input.variants.len());
        for variant in input.variants {
            let value = match variant.value {
                EnumVariantValue::NoValue => None,
                EnumVariantValue::Value(value) => Some(value),
                EnumVariantValue::Tuple(_) | EnumVariantValue::Struct(_) => {
                    log.add_warning(ConversionWarning::UnsupportedFeature {
                        feature: format!(
                            "data-carrying variant `{}` on enum `{}`",
                            variant.name, input.name
                        ),
                        resolution: "C# enums cannot carry data; rendered as a plain member"
                            .to_string(),
                    });
                    None
                }
            };

            let name = rename_identifier(
                &config,
                &variant.name,
                TargetLanguage::CSharp,
                IdentifierKind::EnumMember,
            );
            log.add_warnings(name.log.warnings);

            members.push(CSharpEnumMember {
                name: name.value,
                value,
                docs: variant.docs,
            });
        }

        let csharp_enum = CSharpEnum {
            name: name.value,
            visibility: visibility.value,
            underlying_type,
            members,
            is_flags,
            attributes,
            docs: input.docs,
        };
        ConversionResult::with_log(csharp_enum, log)
    }
}

/// Conversion options for C# enums.
#[derive(Debug, Clone, Default)]
pub struct CSharpEnumConversionOptions {
    /// Cross-language conversion configuration (type mapping + renaming).
    pub config: ConversionConfig,
}

/// Variant-level render options for C# enums.
#[derive(Debug, Clone)]
pub struct CSharpEnumVariantRenderOptions {
    /// Whether to render documentation comments on members.
    pub render_docs: bool,
}

impl Default for CSharpEnumVariantRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CSharpEnumVariantRenderOptions {
    pub const DEFAULT: Self = Self { render_docs: true };
}

/// Render options for C# enums.
#[derive(Debug, Clone)]
pub struct CSharpEnumRenderOptions {
    /// Whether to render documentation comments.
    pub render_docs: bool,
    /// Render options for the enum's members.
    pub variant: CSharpEnumVariantRenderOptions,
}

impl Default for CSharpEnumRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CSharpEnumRenderOptions {
    pub const DEFAULT: Self = Self {
        render_docs: true,
        variant: CSharpEnumVariantRenderOptions::DEFAULT,
    };
}
