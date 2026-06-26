use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult},
    convert::{ConversionConfig, IdentifierKind, rename_identifier},
    ir::LanguageNamespace,
    type_map::TargetLanguage,
};

use super::{
    RustConstant, RustConstantRenderOptions, RustDefinition, RustDefinitionRenderOptions, RustEnum,
    RustEnumConversionOptions, RustEnumRenderOptions, RustFunction,
    RustFunctionConversionOptions, RustFunctionRenderOptions, RustStruct, RustStructConversionOptions,
    RustStructRenderOptions, RustVisibility,
};

/// Convert an optional list of native items to their IR form, collecting any warnings.
fn module_items_to_ir<T: BackendItem>(items: Option<Vec<T>>, log: &mut ConversionLog) -> Option<Vec<T::IrType>> {
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
fn module_items_from_ir<T: BackendItem>(
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

/// Represents a Rust module (`mod`).
#[derive(Debug, Clone, PartialEq)]
pub struct RustModule {
    /// The name of the module.
    pub name: String,
    /// The visibility of the module.
    pub visibility: RustVisibility,
    /// The defines lowered into the module.
    pub defines: Option<Vec<RustDefinition>>,
    /// The constants in the module.
    pub constants: Option<Vec<RustConstant>>,
    /// The enums in the module.
    pub enums: Option<Vec<RustEnum>>,
    /// The structs in the module.
    pub structs: Option<Vec<RustStruct>>,
    /// The free functions in the module.
    pub functions: Option<Vec<RustFunction>>,
    /// The submodules in the module.
    pub modules: Option<Vec<RustModule>>,
    /// Optional documentation for the module.
    pub docs: Option<Vec<String>>,
}

impl BackendItem for RustModule {
    type IrType = LanguageNamespace;
    type ConversionOptions = RustModuleConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut log = ConversionLog::new();

        let visibility = self.visibility.to_ir(None);
        log.add_warnings(visibility.log.warnings);

        let language_namespace = LanguageNamespace {
            name: self.name,
            visibility: visibility.value,
            defines: module_items_to_ir(self.defines, &mut log),
            constants: module_items_to_ir(self.constants, &mut log),
            enums: module_items_to_ir(self.enums, &mut log),
            structs: module_items_to_ir(self.structs, &mut log),
            functions: module_items_to_ir(self.functions, &mut log),
            namespaces: module_items_to_ir(self.modules, &mut log),
            docs: self.docs,
        };

        ConversionResult::with_log(language_namespace, log)
    }

    fn from_ir(input: Self::IrType, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();
        let config = options.map(|options| options.config.clone()).unwrap_or_default();

        let visibility = RustVisibility::from_ir(input.visibility, None);
        log.add_warnings(visibility.log.warnings);

        let name = {
            let renamed = rename_identifier(&config, &input.name, TargetLanguage::Rust, IdentifierKind::Namespace);
            log.add_warnings(renamed.log.warnings);
            renamed.value
        };

        let enum_options = RustEnumConversionOptions { config: config.clone() };
        let struct_options = RustStructConversionOptions { config: config.clone() };
        let function_options = RustFunctionConversionOptions { config: config.clone() };
        let module_options = RustModuleConversionOptions { config: config.clone() };

        let rust_module = RustModule {
            name,
            visibility: visibility.value,
            defines: module_items_from_ir::<RustDefinition>(input.defines, None, &mut log),
            constants: module_items_from_ir::<RustConstant>(input.constants, None, &mut log),
            enums: module_items_from_ir::<RustEnum>(input.enums, Some(&enum_options), &mut log),
            structs: module_items_from_ir::<RustStruct>(input.structs, Some(&struct_options), &mut log),
            functions: module_items_from_ir::<RustFunction>(input.functions, Some(&function_options), &mut log),
            modules: module_items_from_ir::<RustModule>(input.namespaces, Some(&module_options), &mut log),
            docs: input.docs,
        };

        ConversionResult::with_log(rust_module, log)
    }
}

/// Conversion options for Rust modules.
#[derive(Debug, Clone, Default)]
pub struct RustModuleConversionOptions {
    /// Cross-language conversion configuration (type mapping + renaming).
    pub config: ConversionConfig,
}

/// Render options for Rust modules.
#[derive(Debug, Clone, Default)]
pub struct RustModuleRenderOptions {
    pub define_options: RustDefinitionRenderOptions,
    pub constant_options: RustConstantRenderOptions,
    pub enum_options: RustEnumRenderOptions,
    pub struct_options: RustStructRenderOptions,
    pub function_options: RustFunctionRenderOptions,
}
