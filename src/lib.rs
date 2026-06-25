//! langprint: a multi-language source-declaration code-generation library.
//!
//! Each typed backend (C++, Rust, C#) owns a rich, full-power native model that it builds and
//! renders directly. The near-untyped backends (Python, Lua, JS) are thin, render-only targets.
//! A neutral declaration intermediate representation (IR) — covering types, fields,
//! enum/function signatures, visibility, namespaces, annotations, and docs — acts as an optional,
//! lossy bridge for cross-language conversion: `to_ir` reports every feature it cannot carry, and
//! `from_ir` lowers the IR into each target language's idioms. Single-language use never touches
//! the IR.
//!
//! Map placement follows one rule: cross-language mapping tables ([`TypeMap`], [`ImportMap`],
//! [`NamingMap`], [`KeywordMap`], [`AnnotationMap`]) are re-exported at the crate root.

pub mod backends;
pub mod conversion;
pub mod convert;
mod helper;
pub mod imports;
pub mod ir;
pub mod naming;
pub mod project_gen;
pub mod renderers;
pub mod text;
pub mod type_map;

pub use convert::{AnnotationMap, CaseStyle, ConversionConfig, KeywordMap, NamingMap};
pub use ir::{Annotation, AnnotationKind};
pub use imports::{ImportEntry, ImportMap, ImportSet};
pub use type_map::{PrimitiveType, TargetLanguage, TypeMap};

/// Available backend names.
pub const AVAILABLE_BACKENDS: &[&str] = &["C++", "Rust", "C#", "Python", "Lua", "JS"];
