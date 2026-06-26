use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    convert::{ConversionConfig, IdentifierKind, rename_identifier},
    ir::{LanguageNamespace, Visibility},
    type_map::TargetLanguage,
};

use super::{
    CSharpConstant, CSharpConstantRenderOptions, CSharpDefinition, CSharpDefinitionRenderOptions, CSharpEnum,
    CSharpEnumConversionOptions, CSharpEnumRenderOptions, CSharpType,
    CSharpTypeConversionOptions, CSharpTypeRenderOptions,
};

/// Convert an optional list of native items to their IR form, collecting any warnings.
fn items_to_ir<T: BackendItem>(items: Option<Vec<T>>, log: &mut ConversionLog) -> Option<Vec<T::IrType>> {
    items.map(|items| {
        items
            .into_iter()
            .map(|item| {
                let result = item.to_ir(None);
                log.add_warnings(result.log.warnings);
                result.value
            })
            .collect()
    })
}

/// Convert an optional list of IR items to their native form, threading conversion options and
/// collecting any warnings.
fn items_from_ir<T: BackendItem>(
    items: Option<Vec<T::IrType>>,
    options: Option<&T::ConversionOptions>,
    log: &mut ConversionLog,
) -> Option<Vec<T>> {
    items.map(|items| {
        items
            .into_iter()
            .map(|item| {
                let result = T::from_ir(item, options);
                log.add_warnings(result.log.warnings);
                result.value
            })
            .collect()
    })
}

/// Represents a C# namespace.
#[derive(Debug, Clone, PartialEq)]
pub struct CSharpNamespace {
    /// The name of the namespace.
    pub name: String,
    /// The defines in the namespace.
    pub defines: Option<Vec<CSharpDefinition>>,
    /// The constants in the namespace.
    pub constants: Option<Vec<CSharpConstant>>,
    /// The enums in the namespace.
    pub enums: Option<Vec<CSharpEnum>>,
    /// The types (classes/structs/interfaces/records) in the namespace.
    pub types: Option<Vec<CSharpType>>,
    /// Nested namespaces.
    pub namespaces: Option<Vec<CSharpNamespace>>,
    /// Whether the namespace is rendered file-scoped (`namespace X;`).
    pub file_scoped: bool,
    /// Documentation for the namespace.
    pub docs: Option<Vec<String>>,
}

impl BackendItem for CSharpNamespace {
    type IrType = LanguageNamespace;
    type ConversionOptions = CSharpNamespaceConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut log = ConversionLog::new();

        if self.file_scoped {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("file-scoped namespace `{}`", self.name),
                resolution: "file-scoped flag dropped from the language-agnostic IR".to_string(),
            });
        }

        let language_namespace = LanguageNamespace {
            name: self.name,
            visibility: Visibility::Default,
            defines: items_to_ir(self.defines, &mut log),
            constants: items_to_ir(self.constants, &mut log),
            enums: items_to_ir(self.enums, &mut log),
            structs: items_to_ir(self.types, &mut log),
            functions: None,
            namespaces: items_to_ir(self.namespaces, &mut log),
            docs: self.docs,
        };
        ConversionResult::with_log(language_namespace, log)
    }

    fn from_ir(input: Self::IrType, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();
        let config = options.map(|options| options.config.clone()).unwrap_or_default();

        if input.visibility != Visibility::Default {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("visibility on namespace `{}`", input.name),
                resolution: "C# namespaces have no visibility specifier; dropped".to_string(),
            });
        }
        if input.functions.is_some() {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("free functions in namespace `{}`", input.name),
                resolution: "C# has no namespace-level free functions; dropped".to_string(),
            });
        }

        let name = {
            let renamed = rename_identifier(&config, &input.name, TargetLanguage::CSharp, IdentifierKind::Namespace);
            log.add_warnings(renamed.log.warnings);
            renamed.value
        };

        let enum_options = CSharpEnumConversionOptions { config: config.clone() };
        let type_options = CSharpTypeConversionOptions { config: config.clone() };
        let namespace_options = CSharpNamespaceConversionOptions { config: config.clone() };

        let csharp_namespace = CSharpNamespace {
            name,
            defines: items_from_ir::<CSharpDefinition>(input.defines, None, &mut log),
            constants: items_from_ir::<CSharpConstant>(input.constants, None, &mut log),
            enums: items_from_ir::<CSharpEnum>(input.enums, Some(&enum_options), &mut log),
            types: items_from_ir::<CSharpType>(input.structs, Some(&type_options), &mut log),
            namespaces: items_from_ir::<CSharpNamespace>(input.namespaces, Some(&namespace_options), &mut log),
            file_scoped: false,
            docs: input.docs,
        };
        ConversionResult::with_log(csharp_namespace, log)
    }
}

/// Conversion options for C# namespaces.
#[derive(Debug, Clone, Default)]
pub struct CSharpNamespaceConversionOptions {
    /// Cross-language conversion configuration (type mapping + renaming).
    pub config: ConversionConfig,
}

/// Render options for C# namespaces.
#[derive(Debug, Clone, Default)]
pub struct CSharpNamespaceRenderOptions {
    pub define_options: CSharpDefinitionRenderOptions,
    pub constant_options: CSharpConstantRenderOptions,
    pub enum_options: CSharpEnumRenderOptions,
    pub type_options: CSharpTypeRenderOptions,
}
