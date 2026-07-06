//! Rust `trait` declarations and `extern` blocks.
//!
//! These are Rust-native FORM shapes with no target-blind IR analogue (not every language has
//! traits or foreign-function blocks), so — like the Python plain-`class` renderer — they are
//! rendered through backend-native entry points on [`RustBackend`](super::RustBackend) rather
//! than a shared cross-language renderer trait. Each contained item is a bodyless
//! [`RustFunction`] (`body: None`), reusing the function engine for the signature.

use super::{RustFunction, RustGenericArgument, RustVisibility};

/// A Rust `trait` declaration. Its methods are bodyless [`RustFunction`]s (`body: None`), each
/// rendering as a `fn …;` signature.
#[derive(Debug, Clone)]
pub struct RustTrait {
    /// The name of the trait.
    pub name: String,
    /// The visibility of the trait.
    pub visibility: RustVisibility,
    /// Generic parameters of the trait.
    pub generic_args: Vec<RustGenericArgument>,
    /// Supertrait bounds, rendered as `: A + B` after the name (empty for none).
    pub supertraits: Vec<String>,
    /// The trait's method signatures.
    pub methods: Vec<RustFunction>,
    /// Attributes applied to the trait (without the leading `#[`).
    pub attributes: Vec<String>,
    /// Optional documentation for the trait.
    pub docs: Option<Vec<String>>,
}

/// A Rust `extern` block, e.g. `unsafe extern "Rust" { … }`. Its items are bodyless
/// [`RustFunction`]s (`body: None`); the block owns the ABI, so the items carry no `abi` of
/// their own.
#[derive(Debug, Clone)]
pub struct RustExternBlock {
    /// The ABI string, e.g. `"Rust"` or `"C"`.
    pub abi: String,
    /// Whether the block is `unsafe extern`.
    pub is_unsafe: bool,
    /// The foreign function declarations inside the block.
    pub items: Vec<RustFunction>,
    /// Optional documentation for the block.
    pub docs: Option<Vec<String>>,
}
