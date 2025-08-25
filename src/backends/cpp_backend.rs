//! C++ backend for type conversion.

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

pub use backend::{CppBackend, DocsStyle};
pub use constant_types::{CppConstant, CppConstantRenderOptions};
pub use define_types::{CppDefinition, CppDefinitionRenderOptions};
pub use enum_types::{CppEnum, CppEnumRenderOptions, CppEnumVariant, CppEnumVariantRenderOptions};
pub use field_types::{CppField, CppFieldConversionOptions, CppFieldRenderOptions};
pub use function_types::{CppFunction, CppFunctionRenderOptions, CppParameter};
pub use generic_types::CppGenericArgument;
pub use namespace_types::{CppNamespace, CppNamespaceRenderOptions};
pub use struct_types::{CppBase, CppStruct, CppStructRenderOptions};
pub use visibility::CppVisibility;
