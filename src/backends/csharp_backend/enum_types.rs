use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    ir::{EnumVariant, EnumVariantValue, LanguageEnum},
};

use super::CSharpVisibility;

/// Represents a single member of a C# enum.
#[derive(Debug, Clone)]
pub struct CSharpEnumMember {
    /// The name of the member.
    pub name: String,
    /// The explicit value of the member, if any.
    pub value: Option<String>,
    /// Documentation for the member.
    pub docs: Option<Vec<String>>,
}

/// Represents a C# enum.
#[derive(Debug, Clone)]
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

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut log = ConversionLog::new();

        if self.is_flags {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("`[Flags]` on enum `{}`", self.name),
                resolution: "Flags attribute dropped from the language-agnostic IR".to_string(),
            });
        }
        for attribute in &self.attributes {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("attribute `[{}]` on enum `{}`", attribute, self.name),
                resolution: "C# attributes dropped from the language-agnostic IR".to_string(),
            });
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
            })
            .collect();

        let language_enum = LanguageEnum {
            name: self.name,
            visibility: visibility.value,
            variants,
            underlying_type: self.underlying_type,
            docs: self.docs,
        };
        ConversionResult::with_log(language_enum, log)
    }

    fn from_ir(input: Self::IrType, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();

        let visibility = CSharpVisibility::from_ir(input.visibility, None);
        log.add_warnings(visibility.log.warnings);

        let members = input
            .variants
            .into_iter()
            .map(|variant| {
                let value = match variant.value {
                    EnumVariantValue::NoValue => None,
                    EnumVariantValue::Value(value) => Some(value),
                    EnumVariantValue::Tuple(_) | EnumVariantValue::Struct(_) => {
                        log.add_warning(ConversionWarning::UnsupportedFeature {
                            feature: format!("data-carrying variant `{}` on enum `{}`", variant.name, input.name),
                            resolution: "C# enums cannot carry data; rendered as a plain member".to_string(),
                        });
                        None
                    }
                };
                CSharpEnumMember {
                    name: variant.name,
                    value,
                    docs: variant.docs,
                }
            })
            .collect();

        let csharp_enum = CSharpEnum {
            name: input.name,
            visibility: visibility.value,
            underlying_type: input.underlying_type,
            members,
            is_flags: false,
            attributes: Vec::new(),
            docs: input.docs,
        };
        ConversionResult::with_log(csharp_enum, log)
    }
}

/// Conversion options for C# enums.
#[derive(Debug, Clone)]
pub struct CSharpEnumConversionOptions {}

impl Default for CSharpEnumConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CSharpEnumConversionOptions {
    pub const DEFAULT: Self = Self {};
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
}

impl Default for CSharpEnumRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CSharpEnumRenderOptions {
    pub const DEFAULT: Self = Self { render_docs: true };
}
