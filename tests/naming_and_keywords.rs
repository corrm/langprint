//! Tests for the configurable `NamingMap` (case conventions) and `KeywordMap` (reserved-word
//! escaping) carried on `ConversionConfig` and applied by `rename_identifier`.

use langprint::convert::{rename_identifier, ConversionConfig, IdentifierKind};
use langprint::{CaseStyle, KeywordMap, NamingMap, TargetLanguage};

#[test]
fn naming_map_builtin_matches_legacy_conventions() {
    let map = NamingMap::builtin();

    assert_eq!(map.resolve(TargetLanguage::Rust, IdentifierKind::Function), Some(CaseStyle::Snake));
    assert_eq!(map.resolve(TargetLanguage::CSharp, IdentifierKind::Type), Some(CaseStyle::Pascal));
    assert_eq!(map.resolve(TargetLanguage::Js, IdentifierKind::Function), Some(CaseStyle::Camel));
    assert_eq!(map.resolve(TargetLanguage::Python, IdentifierKind::Type), Some(CaseStyle::Pascal));
    assert_eq!(map.resolve(TargetLanguage::Cpp, IdentifierKind::Function), None);
}

#[test]
fn naming_map_override_changes_rename() {
    let mut naming_map = NamingMap::builtin();
    naming_map.insert(TargetLanguage::Python, IdentifierKind::Function, CaseStyle::Pascal);

    let overridden = ConversionConfig {
        naming_map,
        ..ConversionConfig::default()
    };
    let result = rename_identifier(&overridden, "my_func", TargetLanguage::Python, IdentifierKind::Function);
    assert_eq!(result.value, "MyFunc");

    let builtin = ConversionConfig::default();
    let snake = rename_identifier(&builtin, "MyFunc", TargetLanguage::Python, IdentifierKind::Function);
    assert_eq!(snake.value, "my_func");
}

#[test]
fn keyword_escape_python_field() {
    let config = ConversionConfig::default();

    let escaped = rename_identifier(&config, "class", TargetLanguage::Python, IdentifierKind::Field);
    assert_eq!(escaped.value, "class_");
    assert!(escaped.log.has_warnings());

    let untouched = rename_identifier(&config, "count", TargetLanguage::Python, IdentifierKind::Field);
    assert_eq!(untouched.value, "count");
    assert!(!untouched.log.has_warnings());
}

#[test]
fn keyword_escape_rust_and_csharp() {
    // rename off so the literal keyword reaches escaping (case conversion would otherwise
    // re-spell `class` to `Class`, which is no longer reserved).
    let config = ConversionConfig {
        rename: false,
        ..ConversionConfig::default()
    };

    let rust = rename_identifier(&config, "type", TargetLanguage::Rust, IdentifierKind::Field);
    assert_eq!(rust.value, "r#type");

    let csharp = rename_identifier(&config, "class", TargetLanguage::CSharp, IdentifierKind::Field);
    assert_eq!(csharp.value, "@class");
}

#[test]
fn keyword_map_user_extend() {
    let mut keyword_map = KeywordMap::empty();
    keyword_map.insert(TargetLanguage::Python, "mykw");

    let config = ConversionConfig {
        rename: false,
        keyword_map,
        ..ConversionConfig::default()
    };
    let escaped = rename_identifier(&config, "mykw", TargetLanguage::Python, IdentifierKind::Field);
    assert_eq!(escaped.value, "mykw_");
}
