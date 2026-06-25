use crate::backends::BackendItem;
use crate::conversion::ConversionResult;
use crate::ir::LanguageGenericArgument;

/// Represents a Rust generic parameter (type, lifetime, or const generic).
#[derive(Debug, Clone, PartialEq)]
pub struct RustGenericArgument {
    /// The name of the parameter (e.g. `T`, `'a`, `N`).
    pub name: String,
    /// Whether this parameter is a lifetime (`'a`).
    pub is_lifetime: bool,
    /// Optional const-generic type (e.g. `usize` for `const N: usize`); `None` for type/lifetime params.
    pub const_type: Option<String>,
    /// Optional inline bound (e.g. `Display + Debug` for `T: Display + Debug`).
    pub bounds: Option<String>,
    /// Optional default value (e.g. `i32` for `T = i32`).
    pub default_value: Option<String>,
}

/// Keyword used in the neutral IR to mark a Rust lifetime parameter.
const LIFETIME_KEYWORD: &str = "lifetime";
/// Keyword prefix used in the neutral IR to mark a Rust const-generic parameter.
const CONST_KEYWORD_PREFIX: &str = "const ";

/// Conversion options for Rust generic arguments.
#[derive(Debug, Clone, Default)]
pub struct RustGenericArgumentConversionOptions;

impl BackendItem for RustGenericArgument {
    type IrType = LanguageGenericArgument;
    type ConversionOptions = RustGenericArgumentConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let keyword = if self.is_lifetime {
            LIFETIME_KEYWORD.to_string()
        } else if let Some(const_type) = &self.const_type {
            format!("{}{}", CONST_KEYWORD_PREFIX, const_type)
        } else {
            String::new()
        };

        ConversionResult::new(LanguageGenericArgument {
            name: self.name,
            keyword,
            default_value: self.default_value,
            where_clause: self.bounds,
        })
    }

    fn from_ir(input: Self::IrType, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let is_lifetime = input.keyword == LIFETIME_KEYWORD;
        let const_type = input
            .keyword
            .strip_prefix(CONST_KEYWORD_PREFIX)
            .map(|rest| rest.to_string());

        ConversionResult::new(RustGenericArgument {
            name: input.name,
            is_lifetime,
            const_type,
            bounds: input.where_clause,
            default_value: input.default_value,
        })
    }
}

impl RustGenericArgument {
    /// Render this parameter inside an angle-bracket generic list (without the brackets).
    pub(crate) fn render_decl(&self) -> String {
        if self.is_lifetime {
            return format!("'{}", self.name.trim_start_matches('\''));
        }

        let mut out = String::new();
        if let Some(const_type) = &self.const_type {
            out.push_str("const ");
            out.push_str(&self.name);
            out.push_str(": ");
            out.push_str(const_type);
        } else {
            out.push_str(&self.name);
            if let Some(bounds) = &self.bounds {
                out.push_str(": ");
                out.push_str(bounds);
            }
        }
        if let Some(default_value) = &self.default_value {
            out.push_str(" = ");
            out.push_str(default_value);
        }
        out
    }

    /// Render the parameter name only (for use after the type name, e.g. `Foo<'a, T, N>`).
    pub(crate) fn render_use(&self) -> String {
        if self.is_lifetime {
            format!("'{}", self.name.trim_start_matches('\''))
        } else {
            self.name.clone()
        }
    }
}

/// Render a generic parameter list as `<...>`, or an empty string when there are none.
pub(crate) fn render_generic_decls(params: &[RustGenericArgument]) -> String {
    if params.is_empty() {
        return String::new();
    }
    let inner = params
        .iter()
        .map(RustGenericArgument::render_decl)
        .collect::<Vec<_>>()
        .join(", ");
    format!("<{}>", inner)
}

/// Render a generic argument list for a type use site as `<...>`, or empty when there are none.
pub(crate) fn render_generic_uses(params: &[RustGenericArgument]) -> String {
    if params.is_empty() {
        return String::new();
    }
    let inner = params
        .iter()
        .map(RustGenericArgument::render_use)
        .collect::<Vec<_>>()
        .join(", ");
    format!("<{}>", inner)
}
