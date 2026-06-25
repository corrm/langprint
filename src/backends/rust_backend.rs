//! Rust backend for the neutral declaration model.

pub mod attributes;
pub mod backend;
pub mod constant_types;
pub mod define_types;
pub mod enum_types;
pub mod field_types;
pub mod function_types;
pub mod generic_types;
pub mod namespace_types;
pub mod struct_types;
pub mod visibility;

pub use backend::RustBackend;
pub use constant_types::{RustConstant, RustConstantConversionOptions, RustConstantRenderOptions};
pub use define_types::{RustDefinition, RustDefinitionConversionOptions, RustDefinitionRenderOptions};
pub use enum_types::{
    RustEnum, RustEnumConversionOptions, RustEnumRenderOptions, RustEnumVariant, RustEnumVariantRenderOptions,
    RustEnumVariantValue,
};
pub use field_types::{RustField, RustFieldConversionOptions, RustFieldRenderOptions};
pub use function_types::{
    RustFunction, RustFunctionConversionOptions, RustFunctionRenderOptions, RustParameter,
    RustParameterConversionOptions, RustSelfKind,
};
pub use generic_types::RustGenericArgument;
pub use namespace_types::{RustModule, RustModuleConversionOptions, RustModuleRenderOptions};
pub use struct_types::{RustStruct, RustStructConversionOptions, RustStructRenderOptions};
pub use visibility::{RustVisibility, RustVisibilityConversionOptions};
