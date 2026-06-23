use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult},
    ir::LanguageNamespace,
};

use super::{RustConstant, RustDefinition, RustEnum, RustStruct, RustVisibility};

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

/// Convert an optional list of IR items to their native form, collecting any warnings.
fn module_items_from_ir<T: BackendItem>(items: Option<Vec<T::IrType>>, log: &mut ConversionLog) -> Option<Vec<T>> {
    items.map(|items| {
        items
            .into_iter()
            .map(|item| {
                let result = T::from_ir(item, None);
                log.add_warnings(result.log.warnings);
                result.value
            })
            .collect()
    })
}

/// Represents a Rust module (`mod`).
#[derive(Debug, Clone)]
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
            namespaces: module_items_to_ir(self.modules, &mut log),
            docs: self.docs,
        };

        ConversionResult::with_log(language_namespace, log)
    }

    fn from_ir(input: Self::IrType, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();

        let visibility = RustVisibility::from_ir(input.visibility, None);
        log.add_warnings(visibility.log.warnings);

        let rust_module = RustModule {
            name: input.name,
            visibility: visibility.value,
            defines: module_items_from_ir::<RustDefinition>(input.defines, &mut log),
            constants: module_items_from_ir::<RustConstant>(input.constants, &mut log),
            enums: module_items_from_ir::<RustEnum>(input.enums, &mut log),
            structs: module_items_from_ir::<RustStruct>(input.structs, &mut log),
            modules: module_items_from_ir::<RustModule>(input.namespaces, &mut log),
            docs: input.docs,
        };

        ConversionResult::with_log(rust_module, log)
    }
}

/// Conversion options for Rust modules.
#[derive(Debug, Clone)]
pub struct RustModuleConversionOptions {}

impl Default for RustModuleConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl RustModuleConversionOptions {
    pub const DEFAULT: Self = Self {};
}
