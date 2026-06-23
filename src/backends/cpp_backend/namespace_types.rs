use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    ir::{LanguageNamespace, Visibility},
};

use super::{
    CppConstant, CppConstantRenderOptions, CppDefinition, CppDefinitionRenderOptions, CppEnum, CppEnumRenderOptions,
    CppEnumVariantRenderOptions, CppStruct, CppStructRenderOptions,
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

/// Convert an optional list of IR items to their native form, collecting any warnings.
fn namespace_items_from_ir<T: BackendItem>(items: Option<Vec<T::IrType>>, log: &mut ConversionLog) -> Option<Vec<T>> {
    items.map(|items| {
        items
            .into_iter()
            .map(|item| {
                let result = T::from_ir(item, None);
                if result.log.has_warnings() {
                    log.add_warnings(result.log.warnings);
                }
                result.value
            })
            .collect()
    })
}

/// Represents a C++ namespace definition.
#[derive(Debug, Clone)]
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
            namespaces: namespace_items_to_ir(self.namespaces, &mut result_log),
            docs: None,
        };

        ConversionResult::with_log(language_namespace, result_log)
    }

    fn from_ir(input: Self::IrType, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut result_log = ConversionLog::new();

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

        let cpp_namespace = CppNamespace {
            name: input.name,
            defines: namespace_items_from_ir::<CppDefinition>(input.defines, &mut result_log),
            constants: namespace_items_from_ir::<CppConstant>(input.constants, &mut result_log),
            enums: namespace_items_from_ir::<CppEnum>(input.enums, &mut result_log),
            structs: namespace_items_from_ir::<CppStruct>(input.structs, &mut result_log),
            namespaces: namespace_items_from_ir::<CppNamespace>(input.namespaces, &mut result_log),
        };

        ConversionResult::with_log(cpp_namespace, result_log)
    }
}

/// Conversion options for C++ namespaces.
#[derive(Debug, Clone)]
pub struct CppNamespaceConversionOptions {}

impl Default for CppNamespaceConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CppNamespaceConversionOptions {
    pub const DEFAULT: Self = Self {};
}

/// Render options for C++ namespaces.
#[derive(Debug, Clone)]
pub struct CppNamespaceRenderOptions {
    pub define_options: CppDefinitionRenderOptions,
    pub constant_options: CppConstantRenderOptions,
    pub enum_options: CppEnumRenderOptions,
    pub enum_variant_options: CppEnumVariantRenderOptions,
    pub struct_options: CppStructRenderOptions,
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
    };
}
