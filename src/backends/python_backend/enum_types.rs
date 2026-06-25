use crate::{
    backends::BackendItem,
    conversion::{dropped_annotations_warning, ConversionLog, ConversionResult},
    convert::{rename_identifier, ConversionConfig, IdentifierKind},
    ir::{EnumVariant, EnumVariantValue, LanguageEnum, Visibility},
    type_map::TargetLanguage,
};

/// A member of a Python `enum.IntEnum` (`MEMBER = <value>`).
///
/// Values are free-form integer literals as strings — `IntEnum` requires int
/// values, but the backend renders the literal text verbatim.
#[derive(Debug, Clone, PartialEq)]
pub struct PythonEnumMember {
    /// The name of the member.
    pub name: String,
    /// The integer value, rendered verbatim (e.g. `0`, `-1`).
    pub value: String,
}

/// Represents a Python `enum.IntEnum`: `class Name(enum.IntEnum):` with
/// `MEMBER = <value>` lines.
#[derive(Debug, Clone, PartialEq)]
pub struct PythonEnum {
    /// The name of the enum.
    pub name: String,
    /// The members of the enum.
    pub members: Vec<PythonEnumMember>,
    /// Optional docstring, rendered as the first triple-quoted body line.
    pub docstring: Option<String>,
}

impl BackendItem for PythonEnum {
    type IrType = LanguageEnum;
    type ConversionOptions = PythonEnumConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let variants = self
            .members
            .into_iter()
            .map(|member| EnumVariant {
                name: member.name,
                value: EnumVariantValue::Value(member.value),
                docs: None,
            })
            .collect();

        ConversionResult::new(LanguageEnum {
            name: self.name,
            visibility: Visibility::Public,
            variants,
            underlying_type: None,
            docs: self.docstring.map(|docstring| vec![docstring]),
            annotations: Vec::new(),
            raw_attributes: Vec::new(),
        })
    }

    fn from_ir(input: Self::IrType, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();
        let config = options.map(|options| options.config.clone()).unwrap_or_default();

        let name = rename_identifier(&config, &input.name, TargetLanguage::Python, IdentifierKind::Type);
        log.add_warnings(name.log.warnings);

        if !input.annotations.is_empty() || !input.raw_attributes.is_empty() {
            log.add_warning(dropped_annotations_warning(
                input.annotations.len() + input.raw_attributes.len(),
                "enum",
                &input.name,
                "Python",
            ));
        }

        let mut members = Vec::with_capacity(input.variants.len());
        for (index, variant) in input.variants.into_iter().enumerate() {
            let value = match variant.value {
                EnumVariantValue::Value(value) => value,
                EnumVariantValue::NoValue => index.to_string(),
                EnumVariantValue::Tuple(_) | EnumVariantValue::Struct(_) => index.to_string(),
            };
            let member_name = rename_identifier(&config, &variant.name, TargetLanguage::Python, IdentifierKind::EnumMember);
            log.add_warnings(member_name.log.warnings);
            members.push(PythonEnumMember {
                name: member_name.value,
                value,
            });
        }

        ConversionResult::with_log(
            PythonEnum {
                name: name.value,
                members,
                docstring: input.docs.map(|docs| docs.join("\n")),
            },
            log,
        )
    }
}

/// Conversion options for Python enums.
#[derive(Debug, Clone, Default)]
pub struct PythonEnumConversionOptions {
    /// Cross-language conversion configuration.
    pub config: ConversionConfig,
}

/// Render options for Python enum members. `enum.IntEnum` members are bare
/// `NAME = value` assignments with no per-member rendering knobs.
#[derive(Debug, Clone, Default)]
pub struct PythonEnumMemberRenderOptions;

/// Render options for Python enums.
#[derive(Debug, Clone)]
pub struct PythonEnumRenderOptions {
    /// Whether to render the docstring.
    pub render_docstring: bool,
}

impl Default for PythonEnumRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl PythonEnumRenderOptions {
    pub const DEFAULT: Self = Self { render_docstring: true };
}
