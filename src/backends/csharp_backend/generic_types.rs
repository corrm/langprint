use crate::ir::LanguageGenericArgument;

/// Represents a C# generic type parameter (e.g. `T` with an optional `where` constraint).
#[derive(Debug, Clone, PartialEq)]
pub struct CSharpGenericArgument {
    /// The name of the parameter (e.g. `T`).
    pub name: String,
    /// Optional `where` constraint clause body (e.g. `class, new()` for `where T : class, new()`).
    pub constraints: Option<String>,
}

impl CSharpGenericArgument {
    /// Project this C# generic parameter into the neutral IR.
    pub(crate) fn to_ir(&self) -> LanguageGenericArgument {
        LanguageGenericArgument {
            name: self.name.clone(),
            keyword: String::new(),
            default_value: None,
            where_clause: self.constraints.clone(),
        }
    }

    /// Build a C# generic parameter from a neutral IR generic argument.
    pub(crate) fn from_ir(input: &LanguageGenericArgument) -> Self {
        CSharpGenericArgument {
            name: input.name.clone(),
            constraints: input.where_clause.clone(),
        }
    }
}

/// Render a generic parameter list as `<...>`, or an empty string when there are none.
pub(crate) fn render_generic_decls(params: &[CSharpGenericArgument]) -> String {
    if params.is_empty() {
        return String::new();
    }
    let inner = params.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join(", ");
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
