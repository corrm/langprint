use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    convert::{ConversionConfig, IdentifierKind, map_type, rename_identifier},
    ir::{LanguageFunction, LanguageFunctionParameter},
    type_map::TargetLanguage,
};

use super::{CppGenericArgument, CppVisibility};

/// Represents a C++ function parameter.
#[derive(Debug, Clone, PartialEq)]
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
        ConversionResult::new(LanguageFunctionParameter {
            name: self.name,
            param_type: self.param_type,
            default_value: self.default_value,
        })
    }

    fn from_ir(
        input: Self::IrType,
        options: Option<&Self::ConversionOptions>,
    ) -> ConversionResult<Self> {
        let config = options
            .map(|options| options.config.clone())
            .unwrap_or_default();
        let param_type = map_type(&config, &input.param_type, TargetLanguage::Cpp);

        ConversionResult::with_log(
            CppParameter {
                name: input.name,
                param_type: param_type.value,
                default_value: input.default_value,
            },
            param_type.log,
        )
    }
}

/// Conversion options for C++ parameters.
#[derive(Debug, Clone, Default)]
pub struct CppParameterConversionOptions {
    /// Cross-language conversion configuration (type mapping + renaming).
    pub config: ConversionConfig,
}

/// Represents a C++ function definition.
#[derive(Debug, Clone, PartialEq)]
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
    /// Whether the function has C linkage (`extern "C"`).
    pub is_extern_c: bool,
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

    fn to_ir(self, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
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
            let result = template_param.to_ir(None);
            if result.log.has_warnings() {
                result_log.add_warnings(result.log.warnings);
            }
            generic_args.push(result.value);
        }

        // Convert visibility
        let visibility_result = self.visibility.to_ir(None);

        // Collect any warnings from visibility conversion
        if visibility_result.log.has_warnings() {
            result_log.add_warnings(visibility_result.log.warnings);
        }

        // Report C++-only modifiers that the language-agnostic IR cannot represent.
        for (active, feature) in [
            (self.is_const, "`const` member function"),
            (self.is_extern_c, "`extern \"C\"` linkage"),
            (self.is_inline, "`inline` specifier"),
            (self.is_noexcept, "`noexcept` specifier"),
            (self.is_friend, "`friend` function"),
            (self.is_deleted, "`= delete`"),
            (self.is_default, "`= default`"),
        ] {
            if active {
                result_log.add_warning(ConversionWarning::UnsupportedFeature {
                    feature: format!("C++ {feature} on `{}`", self.name),
                    resolution: "modifier dropped from the language-agnostic IR".to_string(),
                });
            }
        }

        let mut lang_function = LanguageFunction {
            name: self.name,
            visibility: visibility_result.value,
            parameters,
            generic_args,
            return_type: self.return_type,
            is_static: self.is_static,
            // In C++, a function with no body that is pure virtual is abstract.
            is_abstract: self.is_pure_virtual,
            is_virtual: self.is_virtual,
            is_override: self.is_override,
            is_final: self.is_final,
            body: self.body.clone(),
            docs: self.docs,
            annotations: Vec::new(),
            raw_attributes: Vec::new(),
        };
        if let Some(hooks) = options.and_then(|options| options.config.hooks.as_ref()) {
            hooks.after_to_ir_function(&mut lang_function);
        }

        ConversionResult::with_log(lang_function, result_log)
    }

    fn from_ir(
        mut input: Self::IrType,
        options: Option<&Self::ConversionOptions>,
    ) -> ConversionResult<Self> {
        let mut result_log = ConversionLog::new();
        let config = options
            .map(|options| options.config.clone())
            .unwrap_or_default();
        if let Some(hooks) = &config.hooks {
            hooks.before_from_ir_function(&mut input);
        }

        // Convert parameters
        let parameter_options = CppParameterConversionOptions {
            config: config.clone(),
        };
        let mut parameters: Vec<CppParameter> = Vec::with_capacity(input.parameters.len());
        for param in &input.parameters {
            let param_result: ConversionResult<CppParameter> =
                CppParameter::from_ir(param.clone(), Some(&parameter_options));

            // Collect any warnings from parameter conversion
            if param_result.log.has_warnings() {
                result_log.add_warnings(param_result.log.warnings);
            }

            parameters.push(param_result.value);
        }

        let return_type = match input.return_type {
            Some(return_type) => {
                let mapped = map_type(&config, &return_type, TargetLanguage::Cpp);
                result_log.add_warnings(mapped.log.warnings);
                Some(mapped.value)
            }
            None => None,
        };

        // Convert generic arguments to template parameters
        let mut template_params = Vec::with_capacity(input.generic_args.len());
        for generic_arg in &input.generic_args {
            let result = CppGenericArgument::from_ir(generic_arg.clone(), None);
            if result.log.has_warnings() {
                result_log.add_warnings(result.log.warnings);
            }
            template_params.push(result.value);
        }

        // Convert visibility
        let visibility_result = CppVisibility::from_ir(input.visibility, None);

        // Collect any warnings from visibility conversion
        if visibility_result.log.has_warnings() {
            result_log.add_warnings(visibility_result.log.warnings);
        }

        for raw in &input.raw_attributes {
            if raw.source != TargetLanguage::Cpp {
                result_log.add_warning(ConversionWarning::UnsupportedFeature {
                    feature: format!("opaque {:?} attribute `{}`", raw.source, raw.text),
                    resolution: "cannot translate to C++; dropped".to_string(),
                });
            }
        }

        let name = rename_identifier(
            &config,
            &input.name,
            TargetLanguage::Cpp,
            IdentifierKind::Function,
        );
        result_log.add_warnings(name.log.warnings);

        let cpp_function = CppFunction {
            name: name.value,
            parent_name: None, // Scope is managed by the caller.
            visibility: visibility_result.value,
            parameters,
            template_params,
            return_type,
            is_static: input.is_static,
            is_const: false,
            // In C++, virtual is determined by is_abstract or is_virtual
            is_virtual: input.is_virtual || input.is_abstract,
            // In C++, an abstract method lowers to a pure virtual function.
            is_pure_virtual: input.is_abstract,
            is_inline: false,
            is_noexcept: false,
            is_extern_c: false,
            is_override: input.is_override,
            is_final: input.is_final,
            is_friend: false,
            is_deleted: false,
            is_default: false,
            body: input.body.clone(),
            docs: input.docs,
        };

        ConversionResult::with_log(cpp_function, result_log)
    }
}

/// Conversion options for C++ functions.
#[derive(Debug, Clone, Default)]
pub struct CppFunctionConversionOptions {
    /// Cross-language conversion configuration (type mapping + renaming).
    pub config: ConversionConfig,
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
    /// Whether to force rendering the function body.
    pub force_render_body: bool,
    /// Whether to render the function body if it's a template.
    pub render_body_if_template: bool,
    /// Whether to render the function body if it's a friend function.
    pub render_body_if_friend: bool,
    /// Whether to emit the `inline` specifier on an out-of-line definition
    /// (`render_definition == true`). Required for member-template definitions
    /// emitted into a header so they don't violate the ODR across translation units.
    pub inline_definition: bool,
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
        force_render_body: false,
        render_body_if_template: true,
        render_body_if_friend: true,
        inline_definition: false,
        use_typename_default: true,
    };
}
