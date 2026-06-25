//! Lua backend for the native declaration model.
//!
//! Lua is the thinnest backend: the language has no types, no visibility, no
//! classes, and no generics. The native model holds only what Lua expresses —
//! a module table (`local M = {}` ... `return M`), untyped functions, and
//! free-form field assignments.

pub mod backend;
pub mod function_types;
pub mod module_types;

pub use backend::LuaBackend;
pub use function_types::{LuaFunction, LuaFunctionConversionOptions, LuaFunctionRenderOptions};
pub use module_types::{LuaField, LuaModule, LuaModuleConversionOptions, LuaModuleRenderOptions};
