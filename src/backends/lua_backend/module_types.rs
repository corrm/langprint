use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    convert::ConversionConfig,
    ir::{LanguageNamespace, Visibility},
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

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut log = ConversionLog::new();

        if !self.fields.is_empty() {
            log.add_warning(ConversionWarning::Other(
                "Lua module field assignments have no IR namespace home; dropping them".to_string(),
            ));
        }

        let mut functions = Vec::with_capacity(self.functions.len());
        for function in self.functions {
            let result = function.to_ir(None);
            log.add_warnings(result.log.warnings);
            functions.push(result.value);
        }

        ConversionResult::with_log(
            LanguageNamespace {
                name: self.table_name,
                visibility: Visibility::Public,
                defines: None,
                constants: None,
                enums: None,
                structs: None,
                functions: Some(functions),
                namespaces: None,
                docs: self.doc.map(|doc| vec![doc]),
            },
            log,
        )
    }

    fn from_ir(input: Self::IrType, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();
        let config = options.map(|options| options.config.clone()).unwrap_or_default();
        let function_options = LuaFunctionConversionOptions { config };

        let mut functions = Vec::new();
        if let Some(input_functions) = input.functions {
            functions.reserve(input_functions.len());
            for function in input_functions {
                let result = LuaFunction::from_ir(function, Some(&function_options));
                log.add_warnings(result.log.warnings);
                functions.push(result.value);
            }
        }

        ConversionResult::with_log(
            LuaModule {
                table_name: input.name,
                fields: Vec::new(),
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
