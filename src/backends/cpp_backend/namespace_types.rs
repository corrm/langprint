use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    convert::{ConversionConfig, IdentifierKind, rename_identifier},
    ir::{LanguageNamespace, Visibility},
    type_map::TargetLanguage,
};

use super::{
    CppConstant, CppConstantRenderOptions, CppDefinition, CppDefinitionRenderOptions, CppEnum, CppEnumConversionOptions,
    CppEnumRenderOptions, CppEnumVariantRenderOptions, CppFunction, CppFunctionConversionOptions,
    CppFunctionRenderOptions, CppStruct, CppStructConversionOptions, CppStructRenderOptions,
};

/// Convert an optional list of native items to their IR form, collecting any warnings.
fn namespace_items_to_ir<T: BackendItem>(items: Option<Vec<T>>, log: &mut ConversionLog) -> Option<Vec<T::IrType>> {
    items.map(|items| {
        items
            .into_iter()
            .map(|item| {
                let result = item.to_ir(None);
                if result.log.has_warnings() {
                    log.add_warnings(result.log.warnings);
                }
                result.value
            })
            .collect()
    })
}

/// Convert an optional list of IR items to their native form, threading conversion options and
/// collecting any warnings.
fn namespace_items_from_ir<T: BackendItem>(
    items: Option<Vec<T::IrType>>,
    options: Option<&T::ConversionOptions>,
    log: &mut ConversionLog,
) -> Option<Vec<T>> {
    items.map(|items| {
        items
            .into_iter()
            .map(|item| {
                let result = T::from_ir(item, options);
                if result.log.has_warnings() {
                    log.add_warnings(result.log.warnings);
                }
                result.value
            })
            .collect()
    })
}

/// Represents a C++ namespace definition.
#[derive(Debug, Clone, PartialEq)]
pub struct CppNamespace {
    /// The name of the namespace.
    pub name: String,
    /// The defines in the namespace.
    pub defines: Option<Vec<CppDefinition>>,
    /// The constants in the namespace.
    pub constants: Option<Vec<CppConstant>>,
    /// The enums in the namespace.
    pub enums: Option<Vec<CppEnum>>,
    /// The structs in the namespace.
    pub structs: Option<Vec<CppStruct>>,
    /// The free functions in the namespace.
    pub functions: Option<Vec<CppFunction>>,
    /// The namespaces in the namespace.
    pub namespaces: Option<Vec<CppNamespace>>,
}

impl BackendItem for CppNamespace {
    type IrType = LanguageNamespace;
    type ConversionOptions = CppNamespaceConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut result_log = ConversionLog::new();

        let language_namespace = LanguageNamespace {
            name: self.name,
            visibility: Visibility::Default,
            defines: namespace_items_to_ir(self.defines, &mut result_log),
            constants: namespace_items_to_ir(self.constants, &mut result_log),
            enums: namespace_items_to_ir(self.enums, &mut result_log),
            structs: namespace_items_to_ir(self.structs, &mut result_log),
            functions: namespace_items_to_ir(self.functions, &mut result_log),
            namespaces: namespace_items_to_ir(self.namespaces, &mut result_log),
            docs: None,
        };

        ConversionResult::with_log(language_namespace, result_log)
    }

    fn from_ir(input: Self::IrType, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut result_log = ConversionLog::new();
        let config = options.map(|options| options.config.clone()).unwrap_or_default();

        if input.visibility != Visibility::Default {
            result_log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("visibility on namespace `{}`", input.name),
                resolution: "C++ namespaces have no visibility specifier; dropped".to_string(),
            });
        }
        if input.docs.is_some() {
            result_log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("documentation on namespace `{}`", input.name),
                resolution: "C++ namespaces carry no documentation; dropped".to_string(),
            });
        }

        let name = {
            let renamed = rename_identifier(&config, &input.name, TargetLanguage::Cpp, IdentifierKind::Namespace);
            result_log.add_warnings(renamed.log.warnings);
            renamed.value
        };

        let enum_options = CppEnumConversionOptions {
            config: config.clone(),
            ..Default::default()
        };
        let struct_options = CppStructConversionOptions { config: config.clone() };
        let function_options = CppFunctionConversionOptions { config: config.clone() };
        let namespace_options = CppNamespaceConversionOptions { config: config.clone() };

        let cpp_namespace = CppNamespace {
            name,
            defines: namespace_items_from_ir::<CppDefinition>(input.defines, None, &mut result_log),
            constants: namespace_items_from_ir::<CppConstant>(input.constants, None, &mut result_log),
            enums: namespace_items_from_ir::<CppEnum>(input.enums, Some(&enum_options), &mut result_log),
            structs: namespace_items_from_ir::<CppStruct>(input.structs, Some(&struct_options), &mut result_log),
            functions: namespace_items_from_ir::<CppFunction>(input.functions, Some(&function_options), &mut result_log),
            namespaces: namespace_items_from_ir::<CppNamespace>(
                input.namespaces,
                Some(&namespace_options),
                &mut result_log,
            ),
        };

        ConversionResult::with_log(cpp_namespace, result_log)
    }
}

/// Conversion options for C++ namespaces.
#[derive(Debug, Clone, Default)]
pub struct CppNamespaceConversionOptions {
    /// Cross-language conversion configuration (type mapping + renaming).
    pub config: ConversionConfig,
}

/// Render options for C++ namespaces.
#[derive(Debug, Clone)]
pub struct CppNamespaceRenderOptions {
    pub define_options: CppDefinitionRenderOptions,
    pub constant_options: CppConstantRenderOptions,
    pub enum_options: CppEnumRenderOptions,
    pub enum_variant_options: CppEnumVariantRenderOptions,
    pub struct_options: CppStructRenderOptions,
    pub function_options: CppFunctionRenderOptions,
}

impl Default for CppNamespaceRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CppNamespaceRenderOptions {
    pub const DEFAULT: Self = Self {
        define_options: CppDefinitionRenderOptions::DEFAULT,
        constant_options: CppConstantRenderOptions::DEFAULT,
        enum_options: CppEnumRenderOptions::DEFAULT,
        enum_variant_options: CppEnumVariantRenderOptions::DEFAULT,
        struct_options: CppStructRenderOptions::DEFAULT,
        function_options: CppFunctionRenderOptions::DEFAULT,
    };
}
