use crate::{backends::BackendItem, conversion::ConversionResult, ir::LanguageNamespace};

use super::{
    CppConstant, CppConstantRenderOptions, CppDefinition, CppDefinitionRenderOptions, CppEnum, CppEnumRenderOptions,
    CppEnumVariantRenderOptions, CppStruct, CppStructRenderOptions,
};

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
        todo!()
    }

    fn from_ir(_input: Self::IrType, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        todo!()
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
