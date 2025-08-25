//! LangPrint: A library for converting enums between programming languages
//! using a general intermediate representation (IR).

// Module declarations
pub mod backends;
pub mod conversion;
mod helper;
pub mod ir;
pub mod renderers;
pub mod text;

use backends::BackendItem;
use conversion::ConversionResult;

/// Get a list of available backend names
pub fn available_backends() -> Vec<&'static str> {
    vec!["C++"]
}
