use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    convert::ConversionConfig,
    ir::{LanguageFunction, LanguageFunctionParameter, Visibility},
};

/// Represents a Lua function.
///
/// Lua functions are wholly untyped: parameters carry only names, and there is
/// no return-type or visibility surface. The model holds exactly what Lua
/// expresses and nothing more.
///
/// # Body-slot contract
///
/// Lua has no declaration-only function form — a function always closes with
/// `end`. The uniform `body: Option<Vec<String>>` seam therefore maps as:
/// `None` => an empty function body (`function name(...)` immediately followed
/// by `end`); `Some(lines)` => each line emitted verbatim one indent deeper
/// between the signature and `end`. The consumer owns the line content.
#[derive(Debug, Clone, PartialEq)]
pub struct LuaFunction {
    /// The name of the function (e.g. `greet` or `M.greet`).
    pub name: String,
    /// The parameter names; Lua parameters are untyped.
    pub parameters: Vec<String>,
    /// Optional doc comment, rendered as `-- ...` lines above the function.
    pub doc: Option<String>,
    /// The function body, one entry per line; `None` renders an empty body.
    pub body: Option<Vec<String>>,
}

impl BackendItem for LuaFunction {
    type IrType = LanguageFunction;
    type ConversionOptions = LuaFunctionConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let parameters = self
            .parameters
            .into_iter()
            .map(|name| LanguageFunctionParameter {
                name,
                param_type: String::new(),
                default_value: None,
            })
            .collect();

        ConversionResult::new(LanguageFunction {
            name: self.name,
            visibility: Visibility::Public,
            parameters,
            generic_args: Vec::new(),
            return_type: None,
            is_static: true,
            is_abstract: false,
            is_virtual: false,
            is_override: false,
            is_final: false,
            body: self.body,
            docs: self.doc.map(|doc| vec![doc]),
        })
    }

    fn from_ir(input: Self::IrType, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();

        if !input.generic_args.is_empty() {
            log.add_warning(ConversionWarning::Other(
                "Lua has no generics; dropping generic arguments".to_string(),
            ));
        }
        if input.return_type.is_some() {
            log.add_warning(ConversionWarning::Other(
                "Lua functions are untyped; dropping return type".to_string(),
            ));
        }

        let parameters = input.parameters.into_iter().map(|parameter| parameter.name).collect();

        ConversionResult::with_log(
            LuaFunction {
                name: input.name,
                parameters,
                doc: input.docs.map(|docs| docs.join("\n")),
                body: input.body,
            },
            log,
        )
    }
}

/// Conversion options for Lua functions.
#[derive(Debug, Clone, Default)]
pub struct LuaFunctionConversionOptions {
    /// Cross-language conversion configuration.
    pub config: ConversionConfig,
}

/// Render options for Lua functions.
#[derive(Debug, Clone)]
pub struct LuaFunctionRenderOptions {
    /// Whether to render the doc comment.
    pub render_doc: bool,
}

impl Default for LuaFunctionRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl LuaFunctionRenderOptions {
    pub const DEFAULT: Self = Self { render_doc: true };
}
