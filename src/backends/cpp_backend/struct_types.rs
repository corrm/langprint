use super::{
    CppField, CppFieldConversionOptions, CppFieldRenderOptions, CppFunction, CppFunctionConversionOptions,
    CppFunctionRenderOptions, CppGenericArgument, CppVisibility,
};
use crate::ir::LanguageStructKind;
use crate::type_map::TargetLanguage;
use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    convert::ConversionConfig,
    ir::{Annotation, LanguageBase, LanguageGenericArgument, LanguageStruct, Visibility},
};

/// Represents a base/super with its visibility in C++.
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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

    fn to_ir(self, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
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
            let result = template_param.to_ir(None);
            if result.log.has_warnings() {
                result_log.add_warnings(result.log.warnings);
            }
            generic_args.push(result.value);
        }

        let mut annotations = Vec::new();
        if let Some(alignment) = self.alignment {
            annotations.push(Annotation::Aligned(alignment));
        }

        let mut lang_struct = LanguageStruct {
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
            annotations,
            raw_attributes: Vec::new(),
        };

        if let Some(hooks) = options.and_then(|options| options.config.hooks.as_ref()) {
            hooks.after_to_ir_struct(&mut lang_struct);
        }

        ConversionResult::with_log(lang_struct, result_log)
    }

    fn from_ir(mut input: Self::IrType, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut result_log = ConversionLog::new();
        let config = options.map(|options| options.config.clone()).unwrap_or_default();
        if let Some(hooks) = &config.hooks {
            hooks.before_from_ir_struct(&mut input);
        }

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
            let result = CppGenericArgument::from_ir(generic_arg.clone(), None);
            if result.log.has_warnings() {
                result_log.add_warnings(result.log.warnings);
            }
            template_params.push(result.value);
        }

        // Convert visibility
        let visibility_result: ConversionResult<CppVisibility> = CppVisibility::from_ir(input.visibility, None);

        // Collect any warnings from visibility conversion
        if visibility_result.log.has_warnings() {
            result_log.add_warnings(visibility_result.log.warnings);
        }

        let mut alignment = None;
        for annotation in &input.annotations {
            if let Annotation::Aligned(n) = annotation {
                alignment = Some(*n);
            }
        }
        for raw in &input.raw_attributes {
            if raw.source != TargetLanguage::Cpp {
                result_log.add_warning(ConversionWarning::UnsupportedFeature {
                    feature: format!("opaque {:?} attribute `{}`", raw.source, raw.text),
                    resolution: "cannot translate to C++; dropped".to_string(),
                });
            }
        }

        let cpp_struct = CppStruct {
            struct_kind: match input.struct_kind {
                LanguageStructKind::Struct => CppStructKind::Struct,
                LanguageStructKind::Class => CppStructKind::Class,
                LanguageStructKind::Union => CppStructKind::Union,
            },
            is_final: input.is_final,
            alignment,
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
