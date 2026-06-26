//! Python ctypes type map.
//!
//! Returns a [`TypeMap`] with ctypes spellings for Python output.
//! Use it directly as the type map in [`ConversionConfig`](crate::convert::ConversionConfig):
//!
//! ```
//! use langprint::backends::python_backend::ctypes_type_map;
//! use langprint::convert::ConversionConfig;
//!
//! let config = ConversionConfig::new(ctypes_type_map(), false);
//! ```
//!
//! Custom types (e.g. `MyHandle` → `ctypes.c_void_p`) go through `type_override` on `ConversionConfig`.

use crate::type_map::{PrimitiveType, TargetLanguage, TypeMap};

/// Build a [`TypeMap`] with ctypes spellings for Python output.
///
/// The table covers what ctypes has native types for. [`PrimitiveType::I128`] and
/// [`U128`](PrimitiveType::U128) have no ctypes equivalent, so their Python output is removed: they
/// resolve to nothing and surface as an unmapped-type warning, leaving the consumer to supply a
/// `type_override`. [`PrimitiveType::Void`] maps to `None`, which is correct for a void return.
///
/// Use this as the `type_map` in [`ConversionConfig`](crate::convert::ConversionConfig).
pub fn ctypes_type_map() -> TypeMap {
    let mut map = TypeMap::default();

    for (primitive, spelling) in [
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
    ] {
        map.set_output(primitive, TargetLanguage::Python, spelling);
    }

    map.clear_output(PrimitiveType::I128, TargetLanguage::Python);
    map.clear_output(PrimitiveType::U128, TargetLanguage::Python);

    map
}
