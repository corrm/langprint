//! Body-slot emission contract, locked across every backend.
//!
//! `body: None` renders a bare DECLARATION terminated for the language (`;`, no block).
//! `body: Some(lines)` renders the signature followed by an open block, each consumer
//! line emitted VERBATIM one indent deeper, then the close block. langprint owns only
//! the indentation and block punctuation; it never models statements or expressions.

use langprint::backends::cpp_backend::{
    CppBackend, CppFunction, CppFunctionRenderOptions, CppVisibility, DocsStyle,
};
use langprint::backends::csharp_backend::{CSharpBackend, CSharpMethod, CSharpVisibility};
use langprint::backends::rust_backend::{RustBackend, RustFunction, RustSelfKind, RustVisibility};
use langprint::renderers::FunctionRenderer;
use langprint::text::{IndentStyle, NewLineStyle};

const RAW_LINE: &str = "<RAW BODY LINE: x += 1;>";

/// Asserts the two contract arms on one already-rendered pair:
/// the declaration ends in `;` with no block, the definition wraps the verbatim line in a block.
fn assert_contract(language: &str, declaration: &str, definition: &str) {
    assert!(
        declaration.contains(';'),
        "{language}: `body: None` must terminate the declaration with `;`, got: {declaration:?}"
    );
    assert!(
        !declaration.contains('{') && !declaration.contains('}'),
        "{language}: `body: None` must not emit a block, got: {declaration:?}"
    );
    assert!(
        !declaration.contains(RAW_LINE),
        "{language}: `body: None` must not emit any body line, got: {declaration:?}"
    );

    assert!(
        definition.contains('{') && definition.contains('}'),
        "{language}: `body: Some` must open and close a block, got: {definition:?}"
    );
    assert!(
        definition.contains(RAW_LINE),
        "{language}: `body: Some` must emit the consumer line VERBATIM, got: {definition:?}"
    );
}

#[test]
fn rust_function_honors_body_slot_contract() {
    let backend = RustBackend::default();
    let make = |body: Option<Vec<String>>| RustFunction {
        name: "f".to_string(),
        visibility: RustVisibility::Pub,
        self_kind: RustSelfKind::None,
        parameters: Vec::new(),
        generic_args: Vec::new(),
        return_type: None,
        is_unsafe: false,
        is_async: false,
        is_const: false,
        abi: None,
        body,
        attributes: Vec::new(),
        docs: None,
    };

    let declaration = backend
        .render_function(&make(None), None::<&str>, None::<&str>, None, &mut 0)
        .unwrap();
    let definition = backend
        .render_function(
            &make(Some(vec![RAW_LINE.to_string()])),
            None::<&str>,
            None::<&str>,
            None,
            &mut 0,
        )
        .unwrap();

    assert_contract("Rust", &declaration, &definition);
}

#[test]
fn csharp_method_honors_body_slot_contract() {
    let backend = CSharpBackend::default();
    let make = |body: Option<Vec<String>>| CSharpMethod {
        name: "F".to_string(),
        visibility: CSharpVisibility::Public,
        parameters: Vec::new(),
        generic_args: Vec::new(),
        return_type: None,
        is_static: false,
        is_abstract: false,
        is_virtual: false,
        is_override: false,
        is_sealed: false,
        is_async: false,
        is_unsafe: false,
        body,
        attributes: Vec::new(),
        docs: None,
    };

    let declaration = backend
        .render_function(&make(None), None::<&str>, None::<&str>, None, &mut 0)
        .unwrap();
    let definition = backend
        .render_function(
            &make(Some(vec![RAW_LINE.to_string()])),
            None::<&str>,
            None::<&str>,
            None,
            &mut 0,
        )
        .unwrap();

    assert_contract("C#", &declaration, &definition);
}

#[test]
fn cpp_function_honors_body_slot_contract() {
    let backend = CppBackend {
        new_line: NewLineStyle::LF,
        open_brace_on_new_line: true,
        docs_style: DocsStyle::DoubleSlash,
        indent_style: IndentStyle::Spaces,
        indent_size: 4,
    };
    let make = |body: Option<Vec<String>>| CppFunction {
        name: "f".to_string(),
        parent_name: None,
        visibility: CppVisibility::Public,
        parameters: Vec::new(),
        template_params: Vec::new(),
        return_type: Some("void".to_string()),
        is_static: false,
        is_const: false,
        is_virtual: false,
        is_pure_virtual: false,
        is_inline: false,
        is_noexcept: false,
        is_override: false,
        is_final: false,
        is_extern_c: false,
        is_friend: false,
        is_deleted: false,
        is_default: false,
        body,
        docs: None,
    };

    // C++ gates the body slot behind `render_definition`: with the default options a
    // function renders as a declaration regardless of `body`, so the definition arm is
    // exercised with `render_definition: true`.
    let definition_options = CppFunctionRenderOptions {
        render_definition: true,
        ..CppFunctionRenderOptions::default()
    };

    let declaration = backend
        .render_function(&make(None), None::<&str>, None::<&str>, None, &mut 0)
        .unwrap();
    let definition = backend
        .render_function(
            &make(Some(vec![RAW_LINE.to_string()])),
            None::<&str>,
            None::<&str>,
            Some(&definition_options),
            &mut 0,
        )
        .unwrap();

    assert_contract("C++", &declaration, &definition);
}
