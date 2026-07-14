use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, dropped_annotations_warning},
    convert::{ConversionConfig, IdentifierKind, rename_identifier},
    ir::{EnumVariant, EnumVariantValue, LanguageEnum, Visibility},
    type_map::TargetLanguage,
};

/// A member of a Lua enum table (`Name = <value>,`).
///
/// Values are free-form literals rendered verbatim (e.g. `0`, `1`).
#[derive(Debug, Clone, PartialEq)]
pub struct LuaEnumMember {
    /// The name of the member (the table key).
    pub name: String,
    /// The value, rendered verbatim.
    pub value: String,
}

/// Represents a Lua enum: a plain table literal bound to a local.
///
/// ```lua
/// --- Enum Name
/// local Name = {
///     A = 0,
/// }
/// ```
///
/// Lua has no native enum; this is the idiomatic constant-table form.
#[derive(Debug, Clone, PartialEq)]
pub struct LuaEnum {
    /// The name of the enum (the local table name).
    pub name: String,
    /// The members of the enum.
    pub members: Vec<LuaEnumMember>,
    /// Optional description, rendered as a `--- <doc>` comment above the table.
    pub doc: Option<String>,
}

impl BackendItem for LuaEnum {
    type IrType = LanguageEnum;
    type ConversionOptions = LuaEnumConversionOptions;

    fn to_ir(self, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let variants = self
            .members
            .into_iter()
            .map(|member| EnumVariant {
                name: member.name,
                value: EnumVariantValue::Value(member.value),
                docs: None,
                raw_attributes: Vec::new(),
            })
            .collect();

        let mut ir = LanguageEnum {
            name: self.name,
            visibility: Visibility::Public,
            variants,
            underlying_type: None,
            docs: self.doc.map(|doc| vec![doc]),
            annotations: Vec::new(),
            raw_attributes: Vec::new(),
        };
        if let Some(hooks) = options.and_then(|options| options.config.hooks.as_ref()) {
            hooks.after_to_ir_enum(&mut ir);
        }

        ConversionResult::new(ir)
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
            TargetLanguage::Lua,
            IdentifierKind::Type,
        );
        log.add_warnings(name.log.warnings);

        if !input.annotations.is_empty() || !input.raw_attributes.is_empty() {
            log.add_warning(dropped_annotations_warning(
                input.annotations.len() + input.raw_attributes.len(),
                "enum",
                &input.name,
                "Lua",
            ));
        }

        let mut members = Vec::with_capacity(input.variants.len());
        for (index, variant) in input.variants.into_iter().enumerate() {
            let value = match variant.value {
                EnumVariantValue::Value(value) => value,
                EnumVariantValue::NoValue => index.to_string(),
                EnumVariantValue::Tuple(_) | EnumVariantValue::Struct(_) => index.to_string(),
            };
            let member_name = rename_identifier(
                &config,
                &variant.name,
                TargetLanguage::Lua,
                IdentifierKind::EnumMember,
            );
            log.add_warnings(member_name.log.warnings);
            members.push(LuaEnumMember {
                name: member_name.value,
                value,
            });
        }

        ConversionResult::with_log(
            LuaEnum {
                name: name.value,
                members,
                doc: input.docs.map(|docs| docs.join("\n")),
            },
            log,
        )
    }
}

/// Conversion options for Lua enums.
#[derive(Debug, Clone, Default)]
pub struct LuaEnumConversionOptions {
    /// Cross-language conversion configuration.
    pub config: ConversionConfig,
}

/// Render options for Lua enums.
#[derive(Debug, Clone)]
pub struct LuaEnumRenderOptions {
    /// Whether to render the `--- <doc>` comment above the table.
    pub render_doc: bool,
}

impl Default for LuaEnumRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl LuaEnumRenderOptions {
    pub const DEFAULT: Self = Self { render_doc: true };
}
