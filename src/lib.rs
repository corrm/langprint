//! LangPrint: A library for converting enums between programming languages
//! using a general intermediate representation (IR).

// Module declarations
pub mod backends;
pub mod conversion;
mod helper;
pub mod ir;
pub mod project_gen;
pub mod renderers;
pub mod text;

/// Get a list of available backend names
pub fn available_backends() -> Vec<&'static str> {
    vec!["C++", "Rust", "C#"]
}
