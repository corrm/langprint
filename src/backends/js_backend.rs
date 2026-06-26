//! JavaScript backend for the neutral declaration model.
//!
//! JavaScript is near-untyped: there are no type annotations in the surface
//! syntax (this backend is plain JS, not TypeScript). The native model is
//! deliberately thin and models only what the language actually expresses — a
//! `class` (fields plus methods) and a `function`. Any type information the
//! consumer supplies surfaces only inside an optional JSDoc block, never in the
//! signature.

pub mod backend;
pub mod class_types;
pub mod function_types;

pub use backend::JsBackend;
pub use class_types::{JsClass, JsClassConversionOptions, JsClassRenderOptions, JsField};
pub use function_types::{
    JsFunction, JsFunctionConversionOptions, JsFunctionRenderOptions, JsParameter,
    JsParameterConversionOptions,
};
