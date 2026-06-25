//! Two-tier annotation model for the neutral IR.
//!
//! Native backends carry attributes/derives/repr/alignment that the older IR dropped wholesale.
//! This module preserves them in two tiers:
//!
//! * [`Annotation`] — a closed, curated vocabulary of source-neutral layout facts. A variant is
//!   admitted ONLY when at least two backends each map it to native syntax. The IR stays
//!   target-blind: a variant names a fact ("C layout"), not a particular target's spelling. Adding a
//!   variant is a deliberate IR-contract change, gated on that two-native-mapping rule.
//! * [`RawAttribute`] — an opaque, source-tagged attribute carried verbatim. Single-language
//!   attributes that fail the Tier-1 gate (Rust `derive(...)`, C# `[DllImport(...)]`, …) live here.
//!   They round-trip losslessly within their own language and are dropped (with a warning) when
//!   projected to a different target.

use crate::type_map::TargetLanguage;

/// Tier 1 — curated, source-neutral layout vocabulary.
///
/// Each variant is justified by at least two backends that render it as native syntax. The gate is
/// "at least two native mappings", not "all backends"; a backend with no native form for a concept
/// simply emits nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Annotation {
    /// C-compatible (sequential, standard) memory layout.
    ///
    /// Native mappings: Rust `#[repr(C)]` ↔ C# `[StructLayout(LayoutKind.Sequential)]`. (C++
    /// standard-layout is implicit, so the C++ backend emits nothing.)
    ReprC,
    /// Packed layout with no padding between fields.
    ///
    /// Native mappings: Rust `#[repr(packed)]` ↔ C# `[StructLayout(LayoutKind.Sequential, Pack = 1)]`
    /// ↔ C++ `#pragma pack` / `__attribute__((packed))`.
    Packed,
    /// Minimum alignment of `N` bytes for the type.
    ///
    /// Native mappings: Rust `#[repr(align(N))]` ↔ C++ `alignas(N)`.
    Aligned(u32),
}

/// Tier 2 — an opaque attribute carried verbatim, tagged with its source language.
///
/// `text` is the attribute body in its source syntax, without the language's wrapping punctuation
/// (e.g. `derive(Clone, Debug)`, `DllImport("k")`) — exactly what the native model stores.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawAttribute {
    /// The language the attribute was written in.
    pub source: TargetLanguage,
    /// The attribute body, verbatim, in the source language's syntax.
    pub text: String,
}
