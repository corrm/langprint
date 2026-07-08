//! Per-backend import/using/require management.
//!
//! This is a consumer-driven utility: the consumer builds an [`ImportSet`] for its target language,
//! [`add`](ImportSet::add)s explicit entries or [`add_type_ref`](ImportSet::add_type_ref)s a type
//! name that an [`ImportMap`] resolves to an import, then prepends [`render`](ImportSet::render) to
//! its output. Entries are deduped and deterministically ordered, rendered in the language's native
//! syntax. Backend renderers do not auto-track imports; auto-wiring this into the render paths is a
//! possible future enhancement.
//!
//! [`ImportMap`] mirrors [`TypeMap`](crate::type_map::TypeMap): a [`builtin`](ImportMap::builtin)
//! table of high-confidence, common mappings plus `insert`/`extend`/`clear` so callers curate it.
//!
//! Rendering is additive: an [`ImportSet`] with no entries renders to an empty string.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::type_map::TargetLanguage;

/// A single import the target language needs, carrying exactly what that language's syntax requires.
///
/// One enum spans every backend: the formatter in [`ImportSet::render`] switches on the active
/// [`TargetLanguage`] and reads only the variants that language uses.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImportEntry {
    /// A C++ `#include`. `system` chooses `<...>` (true) over `"..."` (false).
    Include {
        /// The header path, without the surrounding brackets or quotes.
        header: String,
        /// Whether this is a system include (`<...>`) rather than a local one (`"..."`).
        system: bool,
    },
    /// A C# `using` directive: the namespace it imports.
    Using(String),
    /// A Rust `use` path, e.g. `std::collections::HashMap`.
    Use(String),
    /// A Python whole-module import: `import {module}`.
    PyImport(String),
    /// A Python symbol import: `from {module} import {symbol}`.
    PyFrom {
        /// The module to import from.
        module: String,
        /// The symbol imported out of the module.
        symbol: String,
    },
    /// A Lua `local {name} = require("{module}")`.
    Require {
        /// The local binding name.
        name: String,
        /// The module string passed to `require`.
        module: String,
    },
    /// A JS default import: `import {name} from '{source}'`.
    JsDefault {
        /// The bound default name.
        name: String,
        /// The source module specifier.
        source: String,
    },
    /// A JS named import: `import {{ {name} }} from '{source}'`.
    ///
    /// Multiple `JsNamed` entries sharing a `source` render as one grouped
    /// `import {{ a, b, c }} from '{source}'` line (names sorted).
    JsNamed {
        /// The imported binding name.
        name: String,
        /// The source module specifier.
        source: String,
    },
    /// A TypeScript type-only named import: `import type {{ {name} }} from '{source}'`.
    ///
    /// Groups by `source` exactly like [`JsNamed`](ImportEntry::JsNamed), but on
    /// the `import type` line so the binding is erased at emit time.
    JsTypeNamed {
        /// The imported type name.
        name: String,
        /// The source module specifier.
        source: String,
    },
    /// A TypeScript type-only namespace import: `import type * as {alias} from '{source}'`.
    JsTypeNamespace {
        /// The local namespace alias bound to the module's types.
        alias: String,
        /// The source module specifier.
        source: String,
    },
    /// A JS named re-export: `export {{ {name} }} from '{source}'`.
    ///
    /// Groups by `source` like [`JsNamed`](ImportEntry::JsNamed).
    JsReexport {
        /// The re-exported binding name.
        name: String,
        /// The source module specifier.
        source: String,
    },
}

/// Collect `(name, source)` pairs selected from `entries` into a source-keyed,
/// name-sorted map so multiple single-symbol entries render as one grouped line.
fn group_by_source(
    entries: &BTreeSet<ImportEntry>,
    select: impl Fn(&ImportEntry) -> Option<(&str, &str)>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut grouped: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for entry in entries {
        if let Some((name, source)) = select(entry) {
            grouped
                .entry(source.to_string())
                .or_default()
                .insert(name.to_string());
        }
    }
    grouped
}

/// Accumulates [`ImportEntry`] values for one target language and renders them in native syntax.
///
/// Entries are deduped (a [`BTreeSet`]) and rendered in the language's documented order. Construct
/// one per backend output; an empty set renders to `""`.
#[derive(Debug, Clone)]
pub struct ImportSet {
    language: TargetLanguage,
    entries: BTreeSet<ImportEntry>,
}

impl ImportSet {
    /// Create an empty set for `language`.
    pub fn new(language: TargetLanguage) -> Self {
        Self {
            language,
            entries: BTreeSet::new(),
        }
    }

    /// The language this set renders for.
    pub fn language(&self) -> TargetLanguage {
        self.language
    }

    /// Register an import entry. Duplicates collapse to one.
    pub fn add(&mut self, entry: ImportEntry) {
        self.entries.insert(entry);
    }

    /// Resolve a type/symbol reference through `map` and register the import it maps to, if any.
    ///
    /// An unmapped reference registers nothing — no guess, no warning.
    pub fn add_type_ref(&mut self, name: &str, map: &ImportMap) {
        if let Some(entry) = map.resolve(name) {
            self.add(entry.clone());
        }
    }

    /// Whether the set has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Render the import block in the target language's native syntax.
    ///
    /// Returns an empty string when no entries are registered. The trailing newline is included
    /// per non-empty line; the caller prepends the block to its output.
    ///
    /// # Ordering
    ///
    /// * C++ — system `#include <...>` first, then local `#include "..."`; alphabetical within each.
    /// * C# — `using X;` alphabetical by namespace.
    /// * Rust — `use a::b::C;` alphabetical by path.
    /// * Python — `from __future__ import …` first (its symbols merged onto one line), then
    ///   `import x` lines, then `from x import a, b` lines; modules alphabetical, and symbols of
    ///   the same module merged onto one alphabetical `from` line.
    /// * Lua — `local m = require("m")` alphabetical by binding name.
    /// * JS — a fixed kind order: default `import x from '…'`, grouped named `import { a, b } from
    ///   '…'`, grouped type-only `import type { a, b } from '…'`, `import type * as ns from '…'`,
    ///   then grouped `export { a, b } from '…'` re-exports. Entries sharing a source merge onto one
    ///   line; sources alphabetical, names alphabetical within a line.
    pub fn render(&self) -> String {
        let mut lines: Vec<String> = Vec::with_capacity(self.entries.len());
        match self.language {
            TargetLanguage::Cpp => {
                for system in [true, false] {
                    for entry in &self.entries {
                        if let ImportEntry::Include { header, system: s } = entry
                            && *s == system
                        {
                            let line = if *s {
                                format!("#include <{header}>")
                            } else {
                                format!("#include \"{header}\"")
                            };
                            lines.push(line);
                        }
                    }
                }
            }
            TargetLanguage::CSharp => {
                for entry in &self.entries {
                    if let ImportEntry::Using(ns) = entry {
                        lines.push(format!("using {ns};"));
                    }
                }
            }
            TargetLanguage::Rust => {
                for entry in &self.entries {
                    if let ImportEntry::Use(path) = entry {
                        lines.push(format!("use {path};"));
                    }
                }
            }
            TargetLanguage::Python => {
                // `from __future__ import …` is special: Python requires it to be
                // the first statement, ahead of every `import`. Collect its symbols
                // separately and emit them before anything else.
                let mut future: BTreeSet<&str> = BTreeSet::new();
                let mut plain: BTreeSet<&str> = BTreeSet::new();
                let mut froms: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
                for entry in &self.entries {
                    match entry {
                        ImportEntry::PyImport(module) => {
                            plain.insert(module.as_str());
                        }
                        ImportEntry::PyFrom { module, symbol } if module == "__future__" => {
                            future.insert(symbol.as_str());
                        }
                        ImportEntry::PyFrom { module, symbol } => {
                            froms.entry(module.as_str()).or_default().insert(symbol);
                        }
                        _ => {}
                    }
                }
                if !future.is_empty() {
                    let symbols: Vec<&str> = future.into_iter().collect();
                    lines.push(format!("from __future__ import {}", symbols.join(", ")));
                }
                for module in &plain {
                    lines.push(format!("import {module}"));
                }
                for (module, symbols) in &froms {
                    let symbols: Vec<&str> = symbols.iter().copied().collect();
                    lines.push(format!("from {module} import {}", symbols.join(", ")));
                }
            }
            TargetLanguage::Lua => {
                for entry in &self.entries {
                    if let ImportEntry::Require { name, module } = entry {
                        lines.push(format!("local {name} = require(\"{module}\")"));
                    }
                }
            }
            TargetLanguage::Js => {
                // Emit in a fixed kind order: default value imports, grouped named
                // value imports, grouped type-only named imports, type namespace
                // imports, then grouped named re-exports. Named/type-named/re-export
                // entries sharing a source collapse to one `{ a, b }` line.
                for entry in &self.entries {
                    if let ImportEntry::JsDefault { name, source } = entry {
                        lines.push(format!("import {name} from '{source}';"));
                    }
                }
                for (source, names) in group_by_source(&self.entries, |e| match e {
                    ImportEntry::JsNamed { name, source } => Some((name, source)),
                    _ => None,
                }) {
                    let names: Vec<&str> = names.iter().map(String::as_str).collect();
                    lines.push(format!(
                        "import {{ {} }} from '{source}';",
                        names.join(", ")
                    ));
                }
                for (source, names) in group_by_source(&self.entries, |e| match e {
                    ImportEntry::JsTypeNamed { name, source } => Some((name, source)),
                    _ => None,
                }) {
                    let names: Vec<&str> = names.iter().map(String::as_str).collect();
                    lines.push(format!(
                        "import type {{ {} }} from '{source}';",
                        names.join(", ")
                    ));
                }
                for entry in &self.entries {
                    if let ImportEntry::JsTypeNamespace { alias, source } = entry {
                        lines.push(format!("import type * as {alias} from '{source}';"));
                    }
                }
                for (source, names) in group_by_source(&self.entries, |e| match e {
                    ImportEntry::JsReexport { name, source } => Some((name, source)),
                    _ => None,
                }) {
                    let names: Vec<&str> = names.iter().map(String::as_str).collect();
                    lines.push(format!(
                        "export {{ {} }} from '{source}';",
                        names.join(", ")
                    ));
                }
            }
        }

        if lines.is_empty() {
            String::new()
        } else {
            let mut block = lines.join("\n");
            block.push('\n');
            block
        }
    }
}

/// Maps a type/symbol spelling to the import it requires, per language.
///
/// Mirrors [`TypeMap`](crate::type_map::TypeMap): [`builtin`](ImportMap::builtin) seeds the common,
/// high-confidence references and [`insert`](ImportMap::insert)/[`extend`](ImportMap::extend)/
/// [`clear`](ImportMap::clear) let callers curate it. An [`ImportMap`] is single-language: its
/// entries are [`ImportEntry`] values already shaped for one [`TargetLanguage`].
#[derive(Debug, Clone, Default)]
pub struct ImportMap {
    refs: HashMap<String, ImportEntry>,
}

impl ImportMap {
    /// Create an empty map.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create the built-in, high-confidence map for `language`.
    ///
    /// This is the only map `builtin` that takes a [`TargetLanguage`]: its entries are
    /// [`ImportEntry`] values pre-shaped for one language. The other maps are multi-language tables
    /// keyed internally by language, so their `builtin` takes no argument.
    ///
    /// Lua and JS return an empty map: their imports are consumer-driven and inventing builtin
    /// module names would be guessing. Rust returns empty too — primitives need no `use`.
    pub fn builtin(language: TargetLanguage) -> Self {
        let mut map = Self::empty();
        match language {
            TargetLanguage::Cpp => {
                let stdint = ImportEntry::Include {
                    header: "cstdint".to_string(),
                    system: true,
                };
                for name in [
                    "int8_t",
                    "uint8_t",
                    "int16_t",
                    "uint16_t",
                    "int32_t",
                    "uint32_t",
                    "int64_t",
                    "uint64_t",
                    "intptr_t",
                    "uintptr_t",
                ] {
                    map.insert(name, stdint.clone());
                }
                map.insert(
                    "size_t",
                    ImportEntry::Include {
                        header: "cstddef".to_string(),
                        system: true,
                    },
                );
                map.insert(
                    "std::string",
                    ImportEntry::Include {
                        header: "string".to_string(),
                        system: true,
                    },
                );
                map.insert(
                    "std::vector",
                    ImportEntry::Include {
                        header: "vector".to_string(),
                        system: true,
                    },
                );
            }
            TargetLanguage::CSharp => {
                let system = ImportEntry::Using("System".to_string());
                map.insert("IntPtr", system.clone());
                map.insert("UIntPtr", system);
                let interop = ImportEntry::Using("System.Runtime.InteropServices".to_string());
                map.insert("StructLayout", interop.clone());
                map.insert("MarshalAs", interop.clone());
                map.insert("DllImport", interop);
            }
            TargetLanguage::Python => {
                map.insert("ctypes", ImportEntry::PyImport("ctypes".to_string()));
                map.insert(
                    "enum.IntEnum",
                    ImportEntry::PyFrom {
                        module: "enum".to_string(),
                        symbol: "IntEnum".to_string(),
                    },
                );
                map.insert(
                    "enum.Enum",
                    ImportEntry::PyFrom {
                        module: "enum".to_string(),
                        symbol: "Enum".to_string(),
                    },
                );
            }
            TargetLanguage::Rust | TargetLanguage::Lua | TargetLanguage::Js => {}
        }
        map
    }

    /// Look up the import a reference requires.
    pub fn resolve(&self, name: &str) -> Option<&ImportEntry> {
        self.refs.get(name.trim())
    }

    /// Register (or override) the import a reference maps to.
    pub fn insert(&mut self, name: impl Into<String>, entry: ImportEntry) {
        self.refs.insert(name.into(), entry);
    }

    /// Merge another map into this one; entries from `other` take precedence.
    pub fn extend(&mut self, other: ImportMap) {
        self.refs.extend(other.refs);
    }

    /// Remove every mapping.
    pub fn clear(&mut self) {
        self.refs.clear();
    }
}
