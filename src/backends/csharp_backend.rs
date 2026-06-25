//! C# backend for the neutral declaration model.

pub mod attributes;
pub mod backend;
pub mod constant_types;
pub mod define_types;
pub mod enum_types;
pub mod field_types;
pub mod function_types;
pub mod generic_types;
pub mod namespace_types;
pub mod property_types;
pub mod struct_types;
pub mod visibility;

pub use backend::CSharpBackend;
pub use constant_types::{CSharpConstant, CSharpConstantConversionOptions, CSharpConstantRenderOptions};
pub use define_types::{CSharpDefinition, CSharpDefinitionConversionOptions, CSharpDefinitionRenderOptions};
pub use enum_types::{
    CSharpEnum, CSharpEnumConversionOptions, CSharpEnumMember, CSharpEnumRenderOptions, CSharpEnumVariantRenderOptions,
};
pub use field_types::{CSharpField, CSharpFieldConversionOptions, CSharpFieldRenderOptions};
pub use function_types::{
    CSharpMethod, CSharpMethodConversionOptions, CSharpMethodRenderOptions, CSharpParameter,
    CSharpParameterConversionOptions,
};
pub use generic_types::CSharpGenericArgument;
pub use namespace_types::{CSharpNamespace, CSharpNamespaceConversionOptions, CSharpNamespaceRenderOptions};
pub use property_types::CSharpProperty;
pub use struct_types::{CSharpType, CSharpTypeConversionOptions, CSharpTypeKind, CSharpTypeRenderOptions};
pub use visibility::{CSharpVisibility, CSharpVisibilityConversionOptions};
