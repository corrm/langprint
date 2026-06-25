//! Python backend for the neutral declaration model.
//!
//! Python is near-untyped: there is no visibility, no generics, and no access
//! modifiers in the surface syntax. The native model is deliberately thin and
//! models only what the language actually expresses — a `class`, a ctypes
//! `Structure`, an `enum.IntEnum`, and a `def` with optional type hints.

pub mod backend;
pub mod class_types;
pub mod enum_types;
pub mod function_types;
pub mod struct_types;

pub use backend::PythonBackend;
pub use class_types::{
    PythonClass, PythonClassConversionOptions, PythonClassField, PythonClassRenderOptions,
};
pub use enum_types::{
    PythonEnum, PythonEnumConversionOptions, PythonEnumMember, PythonEnumMemberRenderOptions, PythonEnumRenderOptions,
};
pub use function_types::{
    PythonFunction, PythonFunctionConversionOptions, PythonFunctionRenderOptions, PythonParameter,
    PythonParameterConversionOptions,
};
pub use struct_types::{
    PythonStruct, PythonStructConversionOptions, PythonStructField, PythonStructRenderOptions,
};
