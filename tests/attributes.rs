use langprint::{
    backends::rust_backend::{
        RustBackend, RustFunction, RustParameter, RustSelfKind, RustVisibility,
    },
    ir::{AttributeSite, RawAttribute, render_raw_attributes},
    renderers::FunctionRenderer,
    type_map::TargetLanguage,
};

fn attributes(language: TargetLanguage) -> Vec<RawAttribute> {
    vec![
        RawAttribute {
            source: language,
            text: "first".to_string(),
        },
        RawAttribute {
            source: language,
            text: "second".to_string(),
        },
    ]
}

#[test]
fn every_backend_wraps_each_attribute_entry_independently() {
    let cases = [
        (
            TargetLanguage::Rust,
            AttributeSite::Type,
            "#[first]",
            "#[second]",
        ),
        (
            TargetLanguage::Cpp,
            AttributeSite::Field,
            "[[first]]",
            "[[second]]",
        ),
        (
            TargetLanguage::CSharp,
            AttributeSite::Function,
            "[first]",
            "[second]",
        ),
        (
            TargetLanguage::Python,
            AttributeSite::Function,
            "@first",
            "@second",
        ),
        (
            TargetLanguage::Lua,
            AttributeSite::Variant,
            "---@langprint Variant: first",
            "---@langprint Variant: second",
        ),
        (
            TargetLanguage::Js,
            AttributeSite::Parameter,
            "/** @langprint Parameter: first */",
            "/** @langprint Parameter: second */",
        ),
    ];

    for (language, site, first, second) in cases {
        assert_eq!(
            render_raw_attributes(language, site, &attributes(language)),
            [first, second]
        );
    }
}

#[test]
fn backend_matrix_covers_every_authored_declaration_site() {
    let languages = [
        TargetLanguage::Rust,
        TargetLanguage::Cpp,
        TargetLanguage::CSharp,
        TargetLanguage::Python,
        TargetLanguage::Lua,
        TargetLanguage::Js,
    ];
    let sites = [
        AttributeSite::Root,
        AttributeSite::Module,
        AttributeSite::Type,
        AttributeSite::Field,
        AttributeSite::Enum,
        AttributeSite::Variant,
        AttributeSite::Function,
        AttributeSite::Parameter,
        AttributeSite::Return,
    ];

    for language in languages {
        for site in sites {
            let rendered = render_raw_attributes(language, site, &attributes(language));
            assert_eq!(rendered.len(), 2, "{language:?} {site:?}");
            assert!(!rendered[0].is_empty(), "{language:?} {site:?}");
            assert!(!rendered[1].is_empty(), "{language:?} {site:?}");
        }
    }
}

#[test]
fn sites_with_special_grammar_keep_their_identity() {
    assert_eq!(
        render_raw_attributes(
            TargetLanguage::Rust,
            AttributeSite::Root,
            &attributes(TargetLanguage::Rust)
        ),
        ["#![first]", "#![second]"]
    );
    assert_eq!(
        render_raw_attributes(
            TargetLanguage::CSharp,
            AttributeSite::Return,
            &attributes(TargetLanguage::CSharp)
        ),
        ["[return: first]", "[return: second]"]
    );
    assert_eq!(
        render_raw_attributes(
            TargetLanguage::Cpp,
            AttributeSite::Return,
            &attributes(TargetLanguage::Cpp)
        ),
        ["[[first]]", "[[second]]"]
    );
    assert_eq!(
        render_raw_attributes(
            TargetLanguage::CSharp,
            AttributeSite::Root,
            &attributes(TargetLanguage::CSharp)
        ),
        ["[assembly: first]", "[assembly: second]"]
    );
    assert_eq!(
        render_raw_attributes(
            TargetLanguage::Python,
            AttributeSite::Root,
            &attributes(TargetLanguage::Python)
        ),
        ["# @langprint Root: first", "# @langprint Root: second"]
    );
    assert_eq!(
        render_raw_attributes(
            TargetLanguage::Python,
            AttributeSite::Parameter,
            &attributes(TargetLanguage::Python)
        ),
        [
            "# @langprint Parameter: first",
            "# @langprint Parameter: second"
        ]
    );
}

#[test]
fn empty_and_foreign_attribute_lists_emit_no_bytes() {
    assert!(render_raw_attributes(TargetLanguage::Rust, AttributeSite::Type, &[]).is_empty());
    assert!(
        render_raw_attributes(
            TargetLanguage::Rust,
            AttributeSite::Type,
            &attributes(TargetLanguage::CSharp),
        )
        .is_empty()
    );
}

#[test]
fn rust_parameter_and_return_forms_render_at_valid_sites() {
    let function = RustFunction {
        name: "identity".to_string(),
        visibility: RustVisibility::Pub,
        self_kind: RustSelfKind::None,
        parameters: vec![RustParameter {
            name: "value".to_string(),
            param_type: "i32".to_string(),
            attributes: vec!["allow(unused_variables)".to_string()],
        }],
        generic_args: Vec::new(),
        return_type: Some("i32".to_string()),
        return_attributes: vec!["must_use".to_string()],
        is_unsafe: false,
        is_async: false,
        is_const: false,
        abi: None,
        body: Some(vec!["value".to_string()]),
        attributes: Vec::new(),
        docs: None,
        comments: Vec::new(),
    };

    let rendered = RustBackend::default()
        .render_function(&function, None::<&str>, None::<&str>, None, &mut 0)
        .unwrap();
    assert_eq!(
        rendered,
        "#[must_use]\npub fn identity(#[allow(unused_variables)] value: i32) -> i32 {\n    value\n}\n"
    );
}
