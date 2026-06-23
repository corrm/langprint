//! LangPrint: a multi-language source-declaration code-generation library.
//!
//! Each backend (C++, Rust, C#) owns a rich, full-power native model that it builds and renders
//! directly. A neutral declaration intermediate representation (IR) — covering types, fields,
//! enum/function signatures, visibility, namespaces, and docs — acts as an optional, lossy bridge
//! for cross-language conversion: `to_ir` reports every feature it cannot carry, and `from_ir`
//! lowers the IR into each target language's idioms. Single-language use never touches the IR.

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
