use crate::backends::BackendItem;
use crate::conversion::{ConversionLog, ConversionResult, ConversionWarning};
use crate::ir::LanguageGenericArgument;

/// Represents a C# generic type parameter (e.g. `T` with an optional `where` constraint).
#[derive(Debug, Clone, PartialEq)]
pub struct CSharpGenericArgument {
    /// The name of the parameter (e.g. `T`).
    pub name: String,
    /// Optional `where` constraint clause body (e.g. `class, new()` for `where T : class, new()`).
    pub constraints: Option<String>,
}

/// Conversion options for C# generic arguments.
#[derive(Debug, Clone, Default)]
pub struct CSharpGenericArgumentConversionOptions;

impl BackendItem for CSharpGenericArgument {
    type IrType = LanguageGenericArgument;
    type ConversionOptions = CSharpGenericArgumentConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        ConversionResult::new(LanguageGenericArgument {
            name: self.name,
            keyword: String::new(),
            default_value: None,
            where_clause: self.constraints,
        })
    }

    fn from_ir(
        input: Self::IrType,
        _options: Option<&Self::ConversionOptions>,
    ) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();

        if input.default_value.is_some() {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("default on generic parameter `{}`", input.name),
                resolution: "C# has no generic parameter defaults; the default was dropped"
                    .to_string(),
            });
        }
        if !input.keyword.is_empty() {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("`{}` generic parameter `{}`", input.keyword, input.name),
                resolution: "C# has no lifetime/const-generic parameters; the kind was dropped"
                    .to_string(),
            });
        }

        ConversionResult::with_log(
            CSharpGenericArgument {
                name: input.name,
                constraints: input.where_clause,
            },
            log,
        )
    }
}

/// Render a generic parameter list as `<...>`, or an empty string when there are none.
pub(crate) fn render_generic_decls(params: &[CSharpGenericArgument]) -> String {
    if params.is_empty() {
        return String::new();
    }
    let inner = params
        .iter()
        .map(|p| p.name.clone())
        .collect::<Vec<_>>()
        .join(", ");
    format!("<{}>", inner)
}

/// Render the `where` constraint clauses (each prefixed with a space), or empty when there are none.
pub(crate) fn render_where_clauses(params: &[CSharpGenericArgument]) -> String {
    let mut out = String::new();
    for param in params {
        if let Some(constraints) = &param.constraints {
            out.push_str(&format!(" where {} : {}", param.name, constraints));
        }
    }
    out
}
