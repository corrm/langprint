//! Python ctypes spelling map.
//!
//! `ctypes` is a Python-local FFI vocabulary, not a [`TargetLanguage`](crate::type_map::TargetLanguage),
//! so a neutral [`PrimitiveType`] is re-spelled into ctypes here rather than in the shared
//! [`TypeMap`](crate::type_map::TypeMap) output table. [`CtypeMap`] mirrors that map's shape: a
//! [`builtin`](CtypeMap::builtin) table covers the common primitives, and callers can clone it then
//! add, override, or clear entries before driving a conversion.

use std::collections::HashMap;

use crate::type_map::PrimitiveType;

/// Maps neutral [`PrimitiveType`]s to their Python ctypes spellings (e.g. `ctypes.c_int32`).
///
/// The [`builtin`](CtypeMap::builtin) table covers the primitives ctypes has native types for.
/// Callers who need more (e.g. a 128-bit spelling) clone it and [`insert`](CtypeMap::insert).
#[derive(Debug, Clone, Default)]
pub struct CtypeMap {
    spellings: HashMap<PrimitiveType, String>,
}

impl CtypeMap {
    /// Create an empty map that renders nothing.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create the built-in map covering the primitives ctypes has native spellings for.
    ///
    /// [`PrimitiveType::I128`], [`U128`](PrimitiveType::U128), and [`Void`](PrimitiveType::Void)
    /// are absent — ctypes has no native type for them.
    pub fn builtin() -> Self {
        use PrimitiveType::*;

        let table: &[(PrimitiveType, &str)] = &[
            (Bool, "ctypes.c_bool"),
            (I8, "ctypes.c_int8"),
            (U8, "ctypes.c_uint8"),
            (I16, "ctypes.c_int16"),
            (U16, "ctypes.c_uint16"),
            (I32, "ctypes.c_int32"),
            (U32, "ctypes.c_uint32"),
            (I64, "ctypes.c_int64"),
            (U64, "ctypes.c_uint64"),
            (ISize, "ctypes.c_ssize_t"),
            (USize, "ctypes.c_size_t"),
            (F32, "ctypes.c_float"),
            (F64, "ctypes.c_double"),
            (Char, "ctypes.c_char"),
            (Str, "ctypes.c_char_p"),
        ];

        let mut map = Self::empty();
        for (primitive, spelling) in table {
            map.insert(*primitive, *spelling);
        }
        map
    }

    /// Map a primitive to its ctypes spelling (adds or overrides).
    pub fn insert(&mut self, primitive: PrimitiveType, spelling: impl Into<String>) {
        self.spellings.insert(primitive, spelling.into());
    }

    /// Merge another map into this one; entries from `other` take precedence.
    pub fn extend(&mut self, other: CtypeMap) {
        self.spellings.extend(other.spellings);
    }

    /// Remove every entry.
    pub fn clear(&mut self) {
        self.spellings.clear();
    }

    /// Resolve a primitive to its ctypes spelling, or `None` if the map has no entry for it.
    pub fn resolve(&self, primitive: PrimitiveType) -> Option<&str> {
        self.spellings.get(&primitive).map(String::as_str)
    }
}
