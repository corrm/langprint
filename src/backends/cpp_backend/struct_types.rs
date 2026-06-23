use super::{
    CppField, CppFieldConversionOptions, CppFieldRenderOptions, CppFunction, CppFunctionConversionOptions,
    CppFunctionRenderOptions, CppGenericArgument, CppVisibility,
};
use crate::ir::LanguageStructKind;
use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    convert::ConversionConfig,
    ir::{LanguageBase, LanguageGenericArgument, LanguageStruct, Visibility},
};

/// Represents a base/super with its visibility in C++.
#[derive(Debug, Clone)]
pub struct CppBase {
    /// The name of the base/super.
    pub name: String,
    /// The visibility of the inheritance.
    pub visibility: CppVisibility,
}

/// Represents a C++ struct kind.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CppStructKind {
    Struct,
    Class,
    Union,
}

/// Represents a C++ struct definition.
#[derive(Debug, Clone)]
pub struct CppStruct {
    /// Struct kind.
    pub struct_kind: CppStructKind,
    /// Whether the struct is final.
    pub is_final: bool,
    /// Over-alignment for the struct (`alignas(N)`); `None` = natural alignment.
    pub alignment: Option<u32>,
    /// The name of the struct.
    pub name: String,
    /// Template parameters for the struct/class (C++ templates).
    pub template_params: Vec<CppGenericArgument>,
    /// Base/super classes or structs that this struct inherits from.
    pub bases: Vec<CppBase>,
    /// The fields of the struct.
    pub fields: Vec<CppField>,
    /// The methods of the struct.
    pub methods: Vec<CppFunction>,
    /// Documentation for the struct.
    pub docs: Option<Vec<String>>,
}

impl BackendItem for CppStruct {
    type IrType = LanguageStruct;
    type ConversionOptions = CppStructConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut result_log = ConversionLog::new();

        // Convert fields using CppField's to_ir method
        let mut fields = Vec::with_capacity(self.fields.len());

        for field in self.fields {
            let field_result = field.to_ir(None);

            // Collect any warnings from field conversion
            if field_result.log.has_warnings() {
                result_log.add_warnings(field_result.log.warnings);
            }

            fields.push(field_result.value);
        }

        // A C++ type is abstract iff it declares at least one pure-virtual method.
        let is_abstract = self.methods.iter().any(|method| method.is_pure_virtual);

        // Convert methods
        let mut methods = Vec::with_capacity(self.methods.len());

        for method in self.methods {
            let method_result = method.to_ir(None);

            // Collect any warnings from method conversion
            if method_result.log.has_warnings() {
                result_log.add_warnings(method_result.log.warnings);
            }

            methods.push(method_result.value);
        }

        // Convert bases
        let mut bases: Vec<LanguageBase> = Vec::with_capacity(self.bases.len());

        for base in self.bases {
            let visibility = base.visibility.to_ir(None).value;
            bases.push(LanguageBase {
                name: base.name,
                visibility,
            });
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

        if self.alignment.is_some() {
            result_log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("`alignas` on struct `{}`", self.name),
                resolution: "explicit alignment dropped from the language-agnostic IR".to_string(),
            });
        }

        let lang_struct = LanguageStruct {
            visibility: Visibility::Default,
            struct_kind: match self.struct_kind {
                CppStructKind::Struct => LanguageStructKind::Struct,
                CppStructKind::Class => LanguageStructKind::Class,
                CppStructKind::Union => LanguageStructKind::Union,
            },
            is_abstract,
            is_final: self.is_final,
            name: self.name.clone(),
            generic_args,
            bases,
            fields,
            methods,
            docs: self.docs.clone(),
        };

        ConversionResult::with_log(lang_struct, result_log)
    }

    fn from_ir(input: Self::IrType, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut result_log = ConversionLog::new();
        let config = options.map(|options| options.config.clone()).unwrap_or_default();

        // Convert fields using CppField's from_ir method
        let field_options = CppFieldConversionOptions { config: config.clone() };
        let mut fields = Vec::with_capacity(input.fields.len());

        for field in &input.fields {
            let field_result = CppField::from_ir(field.clone(), Some(&field_options));

            // Collect any warnings from field conversion
            if field_result.log.has_warnings() {
                result_log.add_warnings(field_result.log.warnings);
            }

            fields.push(field_result.value);
        }

        // Convert methods using CppFunction's from_ir method
        let function_options = CppFunctionConversionOptions { config: config.clone() };
        let mut methods = Vec::with_capacity(input.methods.len());

        for method in &input.methods {
            let method_result = CppFunction::from_ir(method.clone(), Some(&function_options));

            // Collect any warnings from method conversion
            if method_result.log.has_warnings() {
                result_log.add_warnings(method_result.log.warnings);
            }

            methods.push(method_result.value);
        }

        // Convert bases
        let mut bases = Vec::with_capacity(input.bases.len());

        for base in input.bases {
            let visibility_result: ConversionResult<CppVisibility> = CppVisibility::from_ir(base.visibility, None);

            // Collect any warnings from visibility conversion
            if visibility_result.log.has_warnings() {
                result_log.add_warnings(visibility_result.log.warnings);
            }

            bases.push(CppBase {
                name: base.name,
                visibility: visibility_result.value,
            });
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
        let visibility_result: ConversionResult<CppVisibility> = CppVisibility::from_ir(input.visibility, None);

        // Collect any warnings from visibility conversion
        if visibility_result.log.has_warnings() {
            result_log.add_warnings(visibility_result.log.warnings);
        }

        let cpp_struct = CppStruct {
            struct_kind: match input.struct_kind {
                LanguageStructKind::Struct => CppStructKind::Struct,
                LanguageStructKind::Class => CppStructKind::Class,
                LanguageStructKind::Union => CppStructKind::Union,
            },
            is_final: input.is_final,
            alignment: None,
            name: input.name.clone(),
            template_params,
            bases,
            fields,
            methods,
            docs: input.docs.clone(),
        };

        ConversionResult::with_log(cpp_struct, result_log)
    }
}

/// Conversion options for C++ structs.
#[derive(Debug, Clone, Default)]
pub struct CppStructConversionOptions {
    /// Cross-language conversion configuration (type mapping + renaming).
    pub config: ConversionConfig,
}

/// Render options for C++ structs.
#[derive(Debug, Clone)]
pub struct CppStructRenderOptions {
    /// Whether to align fields in columns.
    pub align_fields: bool,
    /// Whether to use `typename` as the default keyword for type parameters.
    pub use_typename_default: bool,
    /// Whether to render template parameters.
    pub render_template_params: bool,
    /// Whether to render template parameter keywords (typename/class).
    pub render_template_param_keywords: bool,
    /// Whether to render documentation comments.
    pub render_docs: bool,
    /// Whether to render default visibility (public for struct, private for class).
    pub render_default_visibility: bool,
    /// Render options for fields within the struct.
    pub field_options: CppFieldRenderOptions,
    /// Render options for methods within the struct.
    pub method_options: CppFunctionRenderOptions,
}

impl Default for CppStructRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CppStructRenderOptions {
    pub const DEFAULT: Self = Self {
        align_fields: false,
        use_typename_default: true,
        render_template_params: true,
        render_template_param_keywords: true,
        render_docs: true,
        render_default_visibility: true,
        field_options: CppFieldRenderOptions::DEFAULT,
        method_options: CppFunctionRenderOptions::DEFAULT,
    };
}
