use crate::backends::BackendItem;
use crate::conversion::{ConversionLog, ConversionResult, ConversionWarning};
use crate::ir::LanguageGenericArgument;

/// Represents a C++ template parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct CppGenericArgument {
    /// The name of the template parameter.
    pub name: String,
    /// The keyword that precedes the parameter name.
    /// For C++, this is typically:
    /// - "typename" or "class" for type parameters
    /// - "int", "size_t", etc. for value parameters
    /// - "" (empty string) for parameters without an explicit keyword
    pub keyword: String,
    /// Optional default value for the template parameter.
    /// For example: "int" -> "T = int" or "10" -> "T = 10" in C++
    pub default_value: Option<String>,
}

/// Conversion options for C++ generic arguments.
#[derive(Debug, Clone, Default)]
pub struct CppGenericArgumentConversionOptions;

impl BackendItem for CppGenericArgument {
    type IrType = LanguageGenericArgument;
    type ConversionOptions = CppGenericArgumentConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        ConversionResult::new(LanguageGenericArgument {
            name: self.name,
            keyword: self.keyword,
            default_value: self.default_value,
            where_clause: None,
        })
    }

    fn from_ir(
        input: Self::IrType,
        _options: Option<&Self::ConversionOptions>,
    ) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();

        if input.where_clause.is_some() {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("`where` clause on template parameter `{}`", input.name),
                resolution:
                    "C++ has no `where` clauses on template parameters; the constraint was dropped"
                        .to_string(),
            });
        }

        ConversionResult::with_log(
            CppGenericArgument {
                name: input.name,
                keyword: input.keyword,
                default_value: input.default_value,
            },
            log,
        )
    }
}
