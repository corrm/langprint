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

/// The discriminant of an [`Annotation`], without its payload.
///
/// Keys the per-language forward lowering table ([`crate::convert::AnnotationMap`]): the alignment
/// value lives in the [`Annotation`], not the kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnnotationKind {
    /// See [`Annotation::ReprC`].
    ReprC,
    /// See [`Annotation::Packed`].
    Packed,
    /// See [`Annotation::Aligned`].
    Aligned,
}

impl Annotation {
    /// The [`AnnotationKind`] of this annotation, dropping any payload.
    pub fn kind(&self) -> AnnotationKind {
        match self {
            Annotation::ReprC => AnnotationKind::ReprC,
            Annotation::Packed => AnnotationKind::Packed,
            Annotation::Aligned(_) => AnnotationKind::Aligned,
        }
    }
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

/// A declaration attachment point for an attribute list.
///
/// Backends own the concrete delimiters. Sites that have no native attribute
/// grammar use the backend's identity-preserving metadata form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeSite {
    Root,
    Module,
    Type,
    Field,
    Enum,
    Variant,
    Function,
    Parameter,
    Return,
}

/// Render source-tagged inner attribute values at a specific declaration site.
///
/// Entries from another source language are intentionally omitted: opaque
/// values are not translatable. Empty inputs emit no bytes.
pub fn render_raw_attributes(
    language: TargetLanguage,
    site: AttributeSite,
    attributes: &[RawAttribute],
) -> Vec<String> {
    attributes
        .iter()
        .filter(|attribute| attribute.source == language)
        .map(|attribute| render_raw_attribute(language, site, &attribute.text))
        .collect()
}

fn render_raw_attribute(language: TargetLanguage, site: AttributeSite, value: &str) -> String {
    match language {
        TargetLanguage::Rust => match site {
            AttributeSite::Root => format!("#![{value}]"),
            _ => format!("#[{value}]"),
        },
        TargetLanguage::Cpp => match site {
            AttributeSite::Root => format!("// [[langprint::root({value})]]"),
            _ => format!("[[{value}]]"),
        },
        TargetLanguage::CSharp => match site {
            AttributeSite::Root => format!("[assembly: {value}]"),
            AttributeSite::Module => format!("[module: {value}]"),
            AttributeSite::Return => format!("[return: {value}]"),
            _ => format!("[{value}]"),
        },
        TargetLanguage::Python => match site {
            AttributeSite::Type | AttributeSite::Enum | AttributeSite::Function => {
                format!("@{value}")
            }
            _ => format!("# @langprint {site:?}: {value}"),
        },
        TargetLanguage::Lua => format!("---@langprint {site:?}: {value}"),
        TargetLanguage::Js => format!("/** @langprint {site:?}: {value} */"),
    }
}
