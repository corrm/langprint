use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, dropped_annotations_warning},
    convert::{ConversionConfig, IdentifierKind, rename_identifier},
    ir::{EnumVariant, EnumVariantValue, LanguageEnum, Visibility},
    type_map::TargetLanguage,
};

/// A member of a TypeScript const-object enum (`Name: <value>`).
///
/// Values are free-form literals rendered verbatim (e.g. `0`, `1 << 2`).
#[derive(Debug, Clone, PartialEq)]
pub struct JsEnumMember {
    /// The name of the member.
    pub name: String,
    /// The value, rendered verbatim.
    pub value: String,
}

/// Represents a TypeScript "const object" enum — the runtime-free enum idiom:
///
/// ```ts
/// export const Name = Object.freeze({
///     A: 0,
/// } as const);
/// export type Name = typeof Name[keyof typeof Name];
/// ```
///
/// This avoids the runtime cost of a native `enum` while keeping a usable type.
#[derive(Debug, Clone, PartialEq)]
pub struct JsEnum {
    /// The name of the enum.
    pub name: String,
    /// The members of the enum.
    pub members: Vec<JsEnumMember>,
    /// Optional free-form JSDoc description text (e.g. `Enum LogLevel`).
    pub doc: Option<String>,
    /// `true` to prefix the `const` and companion `type` with `export`.
    pub export: bool,
}

impl BackendItem for JsEnum {
    type IrType = LanguageEnum;
    type ConversionOptions = JsEnumConversionOptions;

    fn to_ir(self, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let variants = self
            .members
            .into_iter()
            .map(|member| EnumVariant {
                name: member.name,
                value: EnumVariantValue::Value(member.value),
                docs: None,
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
            TargetLanguage::Js,
            IdentifierKind::Type,
        );
        log.add_warnings(name.log.warnings);

        if !input.annotations.is_empty() || !input.raw_attributes.is_empty() {
            log.add_warning(dropped_annotations_warning(
                input.annotations.len() + input.raw_attributes.len(),
                "enum",
                &input.name,
                "JavaScript",
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
                TargetLanguage::Js,
                IdentifierKind::EnumMember,
            );
            log.add_warnings(member_name.log.warnings);
            members.push(JsEnumMember {
                name: member_name.value,
                value,
            });
        }

        ConversionResult::with_log(
            JsEnum {
                name: name.value,
                members,
                doc: input.docs.map(|docs| docs.join("\n")),
                export: true,
            },
            log,
        )
    }
}

/// Conversion options for JavaScript enums.
#[derive(Debug, Clone, Default)]
pub struct JsEnumConversionOptions {
    /// Cross-language conversion configuration.
    pub config: ConversionConfig,
}

/// Render options for JavaScript enums.
#[derive(Debug, Clone)]
pub struct JsEnumRenderOptions {
    /// Whether to render the `/** ... */` JSDoc line from `doc`.
    pub render_jsdoc: bool,
}

impl Default for JsEnumRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl JsEnumRenderOptions {
    pub const DEFAULT: Self = Self { render_jsdoc: true };
}
