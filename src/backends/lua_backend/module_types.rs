use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, dropped_feature_warning},
    convert::{ConversionConfig, IdentifierKind, rename_identifier},
    ir::{LanguageConstant, LanguageNamespace, Visibility},
    type_map::TargetLanguage,
};

use super::{LuaFunction, LuaFunctionConversionOptions};

/// A Lua module table field assignment (`name = value`).
///
/// Lua field values are free-form, so both the name and the assigned value are
/// modelled as raw strings. The name may be qualified (e.g. `M.version`).
#[derive(Debug, Clone, PartialEq)]
pub struct LuaField {
    /// The field name (e.g. `version` or `M.version`).
    pub name: String,
    /// The assigned value, rendered verbatim (e.g. `"1.0"`, `42`, `{}`).
    pub value: String,
}

/// Represents a Lua module table (`local M = {}` ... `return M`).
///
/// A module holds free functions and field assignments, then returns the table.
/// This is the conventional Lua module shape; it carries no types or visibility.
#[derive(Debug, Clone, PartialEq)]
pub struct LuaModule {
    /// The local table variable name (conventionally `M`).
    pub table_name: String,
    /// Field assignments held by the module table.
    pub fields: Vec<LuaField>,
    /// Functions held by the module table.
    pub functions: Vec<LuaFunction>,
    /// Optional module doc comment, rendered as `-- ...` lines at the top.
    pub doc: Option<String>,
}

impl BackendItem for LuaModule {
    type IrType = LanguageNamespace;
    type ConversionOptions = LuaModuleConversionOptions;

    fn to_ir(self, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut log = ConversionLog::new();
        let config = options.map(|o| o.config.clone()).unwrap_or_default();

        let constants = self
            .fields
            .into_iter()
            .map(|field| {
                let renamed = rename_identifier(
                    &config,
                    &field.name,
                    TargetLanguage::Lua,
                    IdentifierKind::Field,
                );
                LanguageConstant {
                    name: renamed.value,
                    visibility: Visibility::Public,
                    data_type: String::new(),
                    value: field.value,
                    docs: None,
                }
            })
            .collect::<Vec<_>>();

        let function_options = LuaFunctionConversionOptions {
            config: config.clone(),
        };
        let mut functions = Vec::with_capacity(self.functions.len());
        for function in self.functions {
            let result = function.to_ir(Some(&function_options));
            log.add_warnings(result.log.warnings);
            functions.push(result.value);
        }

        ConversionResult::with_log(
            LanguageNamespace {
                name: self.table_name,
                visibility: Visibility::Public,
                defines: None,
                constants: (!constants.is_empty()).then_some(constants),
                enums: None,
                structs: None,
                functions: Some(functions),
                namespaces: None,
                docs: self.doc.map(|doc| vec![doc]),
                raw_attributes: Vec::new(),
            },
            log,
        )
    }

    fn from_ir(
        input: Self::IrType,
        options: Option<&Self::ConversionOptions>,
    ) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();
        let config = options
            .map(|options| options.config.clone())
            .unwrap_or_default();
        let function_options = LuaFunctionConversionOptions {
            config: config.clone(),
        };

        let table_name = rename_identifier(
            &config,
            &input.name,
            TargetLanguage::Lua,
            IdentifierKind::Namespace,
        );
        log.add_warnings(table_name.log.warnings);

        let fields = input
            .constants
            .into_iter()
            .flatten()
            .map(|constant| LuaField {
                name: constant.name,
                value: constant.value,
            })
            .collect();

        let mut functions = Vec::new();
        if let Some(input_functions) = input.functions {
            functions.reserve(input_functions.len());
            for function in input_functions {
                let result = LuaFunction::from_ir(function, Some(&function_options));
                log.add_warnings(result.log.warnings);
                functions.push(result.value);
            }
        }

        if input.enums.is_some_and(|enums| !enums.is_empty()) {
            log.add_warning(dropped_feature_warning("nested enums", &input.name, "Lua"));
        }
        if input.structs.is_some_and(|structs| !structs.is_empty()) {
            log.add_warning(dropped_feature_warning(
                "nested structs",
                &input.name,
                "Lua",
            ));
        }
        if input.defines.is_some_and(|defines| !defines.is_empty()) {
            log.add_warning(dropped_feature_warning("defines", &input.name, "Lua"));
        }
        if input
            .namespaces
            .is_some_and(|namespaces| !namespaces.is_empty())
        {
            log.add_warning(dropped_feature_warning(
                "nested namespaces",
                &input.name,
                "Lua",
            ));
        }

        ConversionResult::with_log(
            LuaModule {
                table_name: table_name.value,
                fields,
                functions,
                doc: input.docs.map(|docs| docs.join("\n")),
            },
            log,
        )
    }
}

/// Conversion options for Lua modules.
#[derive(Debug, Clone, Default)]
pub struct LuaModuleConversionOptions {
    /// Cross-language conversion configuration.
    pub config: ConversionConfig,
}

/// Render options for Lua modules.
#[derive(Debug, Clone)]
pub struct LuaModuleRenderOptions {
    /// Whether to render doc comments.
    pub render_doc: bool,
}

impl Default for LuaModuleRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl LuaModuleRenderOptions {
    pub const DEFAULT: Self = Self { render_doc: true };
}
