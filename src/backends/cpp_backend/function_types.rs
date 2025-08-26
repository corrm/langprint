use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult},
    ir::{LanguageFunction, LanguageFunctionParameter, LanguageGenericArgument},
};

use super::{CppGenericArgument, CppVisibility};

/// Represents a C++ function parameter.
#[derive(Debug, Clone)]
pub struct CppParameter {
    /// The name of the parameter.
    pub name: String,
    /// The type of the parameter.
    pub param_type: String,
    /// Default value for the parameter, if any.
    pub default_value: Option<String>,
}

impl BackendItem for CppParameter {
    type IrType = LanguageFunctionParameter;
    type ConversionOptions = CppParameterConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        todo!()
    }

    fn from_ir(input: Self::IrType, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        todo!()
    }
}

/// Conversion options for C++ parameters.
#[derive(Debug, Clone)]
pub struct CppParameterConversionOptions {}

impl Default for CppParameterConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CppParameterConversionOptions {
    pub const DEFAULT: Self = Self {};
}

/// Represents a C++ function definition.
#[derive(Debug, Clone)]
pub struct CppFunction {
    /// The name of the function.
    pub name: String,
    /// The parent(class or struct) name this function belongs to, if it's a method.
    pub parent_name: Option<String>,
    /// The visibility of the function.
    pub visibility: CppVisibility,
    /// The parameters of the function.
    pub parameters: Vec<CppParameter>,
    /// Template parameters for the function (C++ templates).
    pub template_params: Vec<CppGenericArgument>,
    /// The return type of the function.
    pub return_type: Option<String>,
    /// Whether the function is static.
    pub is_static: bool,
    /// Whether the function is const.
    pub is_const: bool,
    /// Whether the function is virtual.
    pub is_virtual: bool,
    /// Whether the function is pure virtual.
    pub is_pure_virtual: bool,
    /// Whether the function is inline.
    pub is_inline: bool,
    /// Whether the function is noexcept.
    pub is_noexcept: bool,
    /// Whether the function overrides a base class method.
    pub is_override: bool,
    /// Whether the function is final.
    pub is_final: bool,
    /// Whether the function is friend.
    pub is_friend: bool,
    /// Whether the function is deleted (C++11 = delete).
    pub is_deleted: bool,
    /// Whether the function is defaulted (C++11 = default).
    pub is_default: bool,
    /// The function body code, if available. Each string represents a line of code.
    pub body: Option<Vec<String>>,
    /// Documentation for the function.
    pub docs: Option<Vec<String>>,
}

impl BackendItem for CppFunction {
    type IrType = LanguageFunction;
    type ConversionOptions = CppFunctionConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut result_log = ConversionLog::new();

        // Convert parameters
        let mut parameters = Vec::with_capacity(self.parameters.len());
        for param in self.parameters {
            let param_result = param.to_ir(None);

            // Collect any warnings from parameter conversion
            if param_result.log.has_warnings() {
                result_log.add_warnings(param_result.log.warnings);
            }

            parameters.push(param_result.value);
        }

        // Convert template parameters to generic arguments
        let mut generic_args = Vec::with_capacity(self.template_params.len());
        for template_param in self.template_params {
            generic_args.push(LanguageGenericArgument {
                name: template_param.name,
                keyword: template_param.keyword,
                default_value: template_param.default_value,
                where_clause: None, // C++ doesn't use where clauses
            });
        }

        // Convert visibility
        let visibility_result = self.visibility.to_ir(None);

        // Collect any warnings from visibility conversion
        if visibility_result.log.has_warnings() {
            result_log.add_warnings(visibility_result.log.warnings);
        }

        let lang_function = LanguageFunction {
            name: self.name,
            visibility: visibility_result.value,
            parameters,
            generic_args,
            return_type: self.return_type,
            is_static: self.is_static,
            is_const: self.is_const,
            // In C++, a function is abstract if it's pure virtual
            is_abstract: self.is_pure_virtual,
            is_virtual: self.is_virtual,
            is_pure_virtual: self.is_pure_virtual,
            is_inline: self.is_inline,
            is_noexcept: self.is_noexcept,
            is_override: self.is_override,
            is_final: self.is_final,
            is_friend: self.is_friend,
            is_deleted: self.is_deleted,
            is_default: self.is_default,
            body: self.body.clone(),
            docs: self.docs,
        };

        ConversionResult::with_log(lang_function, result_log)
    }

    fn from_ir(input: Self::IrType, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut result_log = ConversionLog::new();

        // Convert parameters
        let mut parameters: Vec<CppParameter> = Vec::with_capacity(input.parameters.len());
        for param in &input.parameters {
            let param_result: ConversionResult<CppParameter> = CppParameter::from_ir(param.clone(), None);

            // Collect any warnings from parameter conversion
            if param_result.log.has_warnings() {
                result_log.add_warnings(param_result.log.warnings);
            }

            parameters.push(param_result.value);
        }

        // Convert generic arguments to template parameters
        let mut template_params = Vec::with_capacity(input.generic_args.len());
        for generic_arg in &input.generic_args {
            template_params.push(CppGenericArgument {
                name: generic_arg.name.clone(),
                keyword: generic_arg.keyword.clone(),
                default_value: generic_arg.default_value.clone(),
            });
        }

        // Convert visibility
        let visibility_result = CppVisibility::from_ir(input.visibility, None);

        // Collect any warnings from visibility conversion
        if visibility_result.log.has_warnings() {
            result_log.add_warnings(visibility_result.log.warnings);
        }

        let cpp_function = CppFunction {
            name: input.name,
            parent_name: None, // Default to None, should be set by the caller if needed
            visibility: visibility_result.value,
            parameters,
            template_params,
            return_type: input.return_type,
            is_static: input.is_static,
            is_const: input.is_const,
            // In C++, virtual is determined by is_abstract or is_virtual
            is_virtual: input.is_virtual || input.is_abstract,
            // In C++, pure virtual is determined by is_abstract
            is_pure_virtual: input.is_pure_virtual || input.is_abstract,
            is_inline: input.is_inline,
            is_noexcept: input.is_noexcept,
            is_override: input.is_override,
            is_final: input.is_final,
            is_friend: input.is_friend,
            is_deleted: input.is_deleted,
            is_default: input.is_default,
            body: input.body.clone(),
            docs: input.docs,
        };

        ConversionResult::with_log(cpp_function, result_log)
    }
}

/// Conversion options for C++ functions.
#[derive(Debug, Clone)]
pub struct CppFunctionConversionOptions {}

impl Default for CppFunctionConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CppFunctionConversionOptions {
    pub const DEFAULT: Self = Self {};
}

/// Render options for C++ functions.
#[derive(Debug, Clone)]
pub struct CppFunctionRenderOptions {
    /// Whether to render the function as a definition (with body) or declaration (without body).
    /// * `true`: Render as a definition with function body (if available)
    /// * `false`: Render as a declaration only (no function body)
    pub render_definition: bool,
    /// Whether to render documentation comments.
    pub render_docs: bool,
    /// Whether to add documentation comments before definitions.
    pub docs_on_definition: bool,
    /// Whether to force rendering the parent name (class or struct) before the function name.
    pub force_render_parent_name: bool,
    /// Whether to force rendering the function body.
    pub force_render_body: bool,
    /// Whether to render the function body if it's a template.
    pub render_body_if_template: bool,
    /// Whether to render the function body if it's a friend function.
    pub render_body_if_friend: bool,
    /// Whether to use `typename` as the default keyword for type parameters.
    pub use_typename_default: bool,
}

impl Default for CppFunctionRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CppFunctionRenderOptions {
    pub const DEFAULT: Self = Self {
        render_definition: false,
        render_docs: true,
        docs_on_definition: false,
        force_render_parent_name: false,
        force_render_body: false,
        render_body_if_template: true,
        render_body_if_friend: true,
        use_typename_default: true,
    };
}
