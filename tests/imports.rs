//! Per-backend import/using/require management: dedup, deterministic order, native syntax, and
//! automatic resolution from a type reference through an [`ImportMap`].

use langprint::{ImportEntry, ImportMap, ImportSet, TargetLanguage};

fn include(header: &str, system: bool) -> ImportEntry {
    ImportEntry::Include {
        header: header.to_string(),
        system,
    }
}

// ---------- C++ ----------

#[test]
fn cpp_dedup_renders_once() {
    let mut set = ImportSet::new(TargetLanguage::Cpp);
    set.add(include("string", true));
    set.add(include("string", true));
    assert_eq!(set.render(), "#include <string>\n");
}

#[test]
fn cpp_system_before_local_then_alphabetical() {
    let mut set = ImportSet::new(TargetLanguage::Cpp);
    set.add(include("vector", true));
    set.add(include("local.h", false));
    set.add(include("cstdint", true));
    set.add(include("a_local.h", false));
    assert_eq!(
        set.render(),
        "#include <cstdint>\n#include <vector>\n#include \"a_local.h\"\n#include \"local.h\"\n"
    );
}

#[test]
fn cpp_auto_from_type_ref() {
    let map = ImportMap::builtin(TargetLanguage::Cpp);
    let mut set = ImportSet::new(TargetLanguage::Cpp);
    set.add_type_ref("uint32_t", &map);
    assert_eq!(set.render(), "#include <cstdint>\n");
}

#[test]
fn cpp_unmapped_ref_renders_nothing() {
    let map = ImportMap::builtin(TargetLanguage::Cpp);
    let mut set = ImportSet::new(TargetLanguage::Cpp);
    set.add_type_ref("MyOwnType", &map);
    assert!(set.is_empty());
    assert_eq!(set.render(), "");
}

// ---------- C# ----------

#[test]
fn csharp_dedup_and_order() {
    let mut set = ImportSet::new(TargetLanguage::CSharp);
    set.add(ImportEntry::Using("System.Text".to_string()));
    set.add(ImportEntry::Using("System".to_string()));
    set.add(ImportEntry::Using("System".to_string()));
    assert_eq!(set.render(), "using System;\nusing System.Text;\n");
}

#[test]
fn csharp_auto_from_type_ref() {
    let map = ImportMap::builtin(TargetLanguage::CSharp);
    let mut set = ImportSet::new(TargetLanguage::CSharp);
    set.add_type_ref("StructLayout", &map);
    set.add_type_ref("IntPtr", &map);
    assert_eq!(
        set.render(),
        "using System;\nusing System.Runtime.InteropServices;\n"
    );
}

// ---------- Rust ----------

#[test]
fn rust_dedup_and_order() {
    let mut set = ImportSet::new(TargetLanguage::Rust);
    set.add(ImportEntry::Use("std::collections::HashMap".to_string()));
    set.add(ImportEntry::Use("std::collections::HashMap".to_string()));
    set.add(ImportEntry::Use("core::mem::size_of".to_string()));
    assert_eq!(
        set.render(),
        "use core::mem::size_of;\nuse std::collections::HashMap;\n"
    );
}

#[test]
fn rust_builtin_is_empty() {
    assert!(
        ImportMap::builtin(TargetLanguage::Rust)
            .resolve("u32")
            .is_none()
    );
}

// ---------- Python ----------

#[test]
fn python_import_before_from_then_alphabetical() {
    let mut set = ImportSet::new(TargetLanguage::Python);
    set.add(ImportEntry::PyFrom {
        module: "enum".to_string(),
        symbol: "IntEnum".to_string(),
    });
    set.add(ImportEntry::PyImport("sys".to_string()));
    set.add(ImportEntry::PyImport("ctypes".to_string()));
    set.add(ImportEntry::PyImport("ctypes".to_string()));
    assert_eq!(
        set.render(),
        "import ctypes\nimport sys\nfrom enum import IntEnum\n"
    );
}

#[test]
fn python_auto_from_type_ref() {
    let map = ImportMap::builtin(TargetLanguage::Python);
    let mut set = ImportSet::new(TargetLanguage::Python);
    set.add_type_ref("ctypes", &map);
    set.add_type_ref("enum.IntEnum", &map);
    assert_eq!(set.render(), "import ctypes\nfrom enum import IntEnum\n");
}

// ---------- Lua ----------

#[test]
fn lua_dedup_and_order() {
    let mut set = ImportSet::new(TargetLanguage::Lua);
    set.add(ImportEntry::Require {
        name: "json".to_string(),
        module: "cjson".to_string(),
    });
    set.add(ImportEntry::Require {
        name: "json".to_string(),
        module: "cjson".to_string(),
    });
    set.add(ImportEntry::Require {
        name: "bit".to_string(),
        module: "bit".to_string(),
    });
    assert_eq!(
        set.render(),
        "local bit = require(\"bit\")\nlocal json = require(\"cjson\")\n"
    );
}

#[test]
fn lua_builtin_is_empty() {
    assert!(
        ImportMap::builtin(TargetLanguage::Lua)
            .resolve("anything")
            .is_none()
    );
}

// ---------- JS ----------

#[test]
fn js_named_and_default_syntax() {
    let mut set = ImportSet::new(TargetLanguage::Js);
    set.add(ImportEntry::JsNamed {
        name: "A".to_string(),
        source: "b".to_string(),
    });
    set.add(ImportEntry::JsNamed {
        name: "A".to_string(),
        source: "b".to_string(),
    });
    set.add(ImportEntry::JsDefault {
        name: "C".to_string(),
        source: "c".to_string(),
    });
    assert_eq!(set.render(), "import C from 'c';\nimport { A } from 'b';\n");
}

#[test]
fn js_builtin_is_empty() {
    assert!(
        ImportMap::builtin(TargetLanguage::Js)
            .resolve("anything")
            .is_none()
    );
}

// ---------- additive guarantee ----------

#[test]
fn empty_set_renders_empty_for_every_language() {
    for lang in [
        TargetLanguage::Cpp,
        TargetLanguage::CSharp,
        TargetLanguage::Rust,
        TargetLanguage::Python,
        TargetLanguage::Lua,
        TargetLanguage::Js,
    ] {
        assert_eq!(ImportSet::new(lang).render(), "");
    }
}
