//! Python ctypes spelling map.
//!
//! `ctypes` is a Python-local FFI vocabulary, not a [`TargetLanguage`](crate::type_map::TargetLanguage),
//! so a neutral [`PrimitiveType`] is re-spelled into ctypes here rather than in the shared
//! [`TypeMap`](crate::type_map::TypeMap) output table. [`CtypeMap`] mirrors that map's shape: a
//! compile-time [`builtin`](CtypeMap::builtin) table covers the common primitives, and callers can
//! clone it then add primitive overrides, map their own custom types, or clear it.

use std::collections::HashMap;

use crate::type_map::PrimitiveType;

/// The compile-time built-in primitive → ctypes table.
///
/// [`PrimitiveType::I128`], [`U128`](PrimitiveType::U128), and [`Void`](PrimitiveType::Void) are
/// absent — ctypes has no native type for them.
pub const BUILTIN_CTYPES: &[(PrimitiveType, &str)] = &[
    (PrimitiveType::Bool, "ctypes.c_bool"),
    (PrimitiveType::I8, "ctypes.c_int8"),
    (PrimitiveType::U8, "ctypes.c_uint8"),
    (PrimitiveType::I16, "ctypes.c_int16"),
    (PrimitiveType::U16, "ctypes.c_uint16"),
    (PrimitiveType::I32, "ctypes.c_int32"),
    (PrimitiveType::U32, "ctypes.c_uint32"),
    (PrimitiveType::I64, "ctypes.c_int64"),
    (PrimitiveType::U64, "ctypes.c_uint64"),
    (PrimitiveType::ISize, "ctypes.c_ssize_t"),
    (PrimitiveType::USize, "ctypes.c_size_t"),
    (PrimitiveType::F32, "ctypes.c_float"),
    (PrimitiveType::F64, "ctypes.c_double"),
    (PrimitiveType::Char, "ctypes.c_char"),
    (PrimitiveType::Str, "ctypes.c_char_p"),
];

/// Re-spells declaration types into Python ctypes spellings (e.g. `ctypes.c_int32`).
///
/// Resolution layers, in precedence order:
/// 1. a custom type mapping ([`insert_type`](CtypeMap::insert_type)) — an arbitrary source spelling
///    (`"MyHandle"`) to an arbitrary ctype (`"ctypes.c_void_p"`), for types that are not primitives;
/// 2. a primitive override ([`insert`](CtypeMap::insert));
/// 3. the compile-time [`BUILTIN_CTYPES`] table, when this map [includes it](CtypeMap::builtin).
///
/// The builtin defaults are `const` data — cloning [`builtin`](CtypeMap::builtin) allocates nothing
/// until you actually add an override.
#[derive(Debug, Clone, Default)]
pub struct CtypeMap {
    include_builtin: bool,
    primitive_overrides: HashMap<PrimitiveType, String>,
    custom_types: HashMap<String, String>,
}

impl CtypeMap {
    /// An empty map that resolves nothing — not even the builtin primitives.
    pub fn empty() -> Self {
        Self::default()
    }

    /// A map backed by the compile-time [`BUILTIN_CTYPES`] table.
    pub fn builtin() -> Self {
        Self {
            include_builtin: true,
            ..Self::default()
        }
    }

    /// Map a primitive to its ctypes spelling (adds or overrides; wins over the builtin table).
    pub fn insert(&mut self, primitive: PrimitiveType, spelling: impl Into<String>) {
        self.primitive_overrides.insert(primitive, spelling.into());
    }

    /// Map a custom, non-primitive source spelling to a ctype (e.g. `"MyHandle"` → `"ctypes.c_void_p"`).
    ///
    /// Takes precedence over primitive resolution, so it can also force a spelling that *would* be
    /// recognized as a primitive onto a different ctype.
    pub fn insert_type(&mut self, spelling: impl Into<String>, ctype: impl Into<String>) {
        self.custom_types.insert(spelling.into(), ctype.into());
    }

    /// Merge another map into this one; entries from `other` take precedence.
    pub fn extend(&mut self, other: CtypeMap) {
        self.include_builtin |= other.include_builtin;
        self.primitive_overrides.extend(other.primitive_overrides);
        self.custom_types.extend(other.custom_types);
    }

    /// Drop every override, custom mapping, and the builtin table — leaving a fully empty map.
    pub fn clear(&mut self) {
        self.include_builtin = false;
        self.primitive_overrides.clear();
        self.custom_types.clear();
    }

    /// Resolve a primitive to its ctypes spelling: overrides first, then the builtin table.
    pub fn resolve(&self, primitive: PrimitiveType) -> Option<&str> {
        if let Some(spelling) = self.primitive_overrides.get(&primitive) {
            return Some(spelling);
        }
        if self.include_builtin {
            return BUILTIN_CTYPES
                .iter()
                .find(|(candidate, _)| *candidate == primitive)
                .map(|(_, spelling)| *spelling);
        }
        None
    }

    /// Resolve a custom, non-primitive source spelling to its mapped ctype, if any.
    pub fn resolve_type(&self, spelling: &str) -> Option<&str> {
        self.custom_types.get(spelling).map(String::as_str)
    }
}
