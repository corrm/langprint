use std::io::{self, Write};

use super::{
    JsClass, JsClassRenderOptions, JsEnum, JsEnumRenderOptions, JsFunction, JsFunctionRenderOptions,
};
use crate::{
    backends::{BackendFeature, BackendMetadata},
    helper::indent,
    renderers::{EnumRenderer, FunctionRenderer},
    text::{IndentStyle, NewLineStyle},
};

/// Backend that renders the neutral declaration model as idiomatic JavaScript source.
///
/// langprint emits form only and JavaScript carries no type annotations in its
/// surface syntax, so the signature is always untyped. Type information, when the
/// consumer supplies it, surfaces only inside an optional JSDoc block. Every
/// function/method whose `body` is `None` renders an empty `{}` block (JavaScript
/// has no declaration-only function form), and a `body` of `Some(lines)` renders
/// each line verbatim one level deeper. The consumer owns the body content.
#[derive(Debug, Clone)]
pub struct JsBackend {
    /// The newline style to use.
    pub new_line: NewLineStyle,
    /// The indentation style to use.
    pub indent_style: IndentStyle,
    /// The number of spaces per indentation level (when using spaces).
    pub indent_size: i32,
}

impl Default for JsBackend {
    fn default() -> Self {
        Self {
            new_line: NewLineStyle::LF,
            indent_style: IndentStyle::Spaces,
            indent_size: 2,
        }
    }
}

impl JsBackend {
    fn indent(&self, level: i32) -> String {
        indent(level, self.indent_size, self.indent_style)
    }

    /// `true` when the function carries genuine type/description info worth a JSDoc block.
    fn has_doc_info(input: &JsFunction) -> bool {
        input.doc.is_some()
            || input.return_type.is_some()
            || input
                .parameters
                .iter()
                .any(|param| param.type_doc.is_some())
    }

    /// Render a JSDoc block from the type information that genuinely exists on the model.
    fn write_jsdoc(
        &self,
        input: &JsFunction,
        indent_level: i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let pad = self.indent(indent_level);
        let nl = self.new_line.as_str();

        write!(out, "{pad}/**{nl}")?;
        if let Some(doc) = &input.doc {
            for line in doc.split('\n') {
                write!(out, "{pad} * {line}{nl}")?;
            }
        }
        for param in &input.parameters {
            if let Some(type_doc) = &param.type_doc {
                write!(out, "{pad} * @param {{{type_doc}}} {}{nl}", param.name)?;
            }
        }
        if let Some(return_type) = &input.return_type {
            write!(out, "{pad} * @returns {{{return_type}}}{nl}")?;
        }
        write!(out, "{pad} */{nl}")
    }

    /// Render a function or method at the given indentation level.
    ///
    /// `as_method` controls the keyword: free functions are prefixed with
    /// `function`, methods omit it (and may carry a leading `static`).
    fn write_function(
        &self,
        input: &JsFunction,
        options: &JsFunctionRenderOptions,
        as_method: bool,
        indent_level: i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let nl = self.new_line.as_str();

        if options.render_jsdoc && Self::has_doc_info(input) {
            self.write_jsdoc(input, indent_level, out)?;
        }

        write!(out, "{}", self.indent(indent_level))?;
        if as_method {
            if input.is_static {
                write!(out, "static ")?;
            }
            write!(out, "{}(", input.name)?;
        } else {
            write!(out, "function {}(", input.name)?;
        }

        let mut first = true;
        for param in &input.parameters {
            if !first {
                write!(out, ", ")?;
            }
            write!(out, "{}", param.name)?;
            if options.typescript
                && let Some(type_annotation) = &param.type_doc
            {
                write!(out, ": {type_annotation}")?;
            }
            if let Some(default) = &param.default {
                write!(out, " = {default}")?;
            }
            first = false;
        }
        write!(out, ")")?;
        if options.typescript
            && let Some(return_type) = &input.return_type
        {
            write!(out, ": {return_type}")?;
        }

        match &input.body {
            None => write!(out, " {{}}{nl}"),
            Some(lines) => {
                write!(out, " {{{nl}")?;
                let body_level = indent_level + 1;
                for line in lines {
                    write!(out, "{}{line}{nl}", self.indent(body_level))?;
                }
                write!(out, "{}}}{nl}", self.indent(indent_level))
            }
        }
    }
}

impl BackendMetadata for JsBackend {
    fn language_name(&self) -> &'static str {
        "JavaScript"
    }

    fn supported_features(&self) -> &'static [BackendFeature] {
        &[
            BackendFeature::Function,
            BackendFeature::Class,
            BackendFeature::Enum,
        ]
    }
}

impl EnumRenderer for JsBackend {
    type EnumType = JsEnum;
    type RenderOptions = JsEnumRenderOptions;

    fn render_enum_to<S: AsRef<str>>(
        &self,
        input: &Self::EnumType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <JsBackend as EnumRenderer>::default_options();
        let options: &JsEnumRenderOptions = options.unwrap_or(&binding);
        let pad = self.indent(*indent_level);
        let nl = self.new_line.as_str();
        let export = if input.export { "export " } else { "" };

        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }
        if options.render_jsdoc
            && let Some(doc) = &input.doc
        {
            write!(out, "{pad}/** {doc} */{nl}")?;
        }
        write!(
            out,
            "{pad}{export}const {} = Object.freeze({{{nl}",
            input.name
        )?;
        for member in &input.members {
            write!(
                out,
                "{}{}: {},{nl}",
                self.indent(*indent_level + 1),
                member.name,
                member.value
            )?;
        }
        write!(out, "{pad}}} as const);{nl}")?;
        write!(
            out,
            "{pad}{export}type {0} = typeof {0}[keyof typeof {0}];{nl}",
            input.name
        )?;
        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }
        Ok(())
    }
}

impl FunctionRenderer for JsBackend {
    type FunctionType = JsFunction;
    type RenderOptions = JsFunctionRenderOptions;

    fn render_function_to<S: AsRef<str>>(
        &self,
        input: &Self::FunctionType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <JsBackend as FunctionRenderer>::default_options();
        let options: &JsFunctionRenderOptions = options.unwrap_or(&binding);

        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }
        self.write_function(input, options, false, *indent_level, out)?;
        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }
        Ok(())
    }
}

impl JsBackend {
    /// Render a `class` (`class Name {` / `class Name extends Base {`) to a writer.
    ///
    /// JavaScript classes are not one of the shared renderer traits' shapes (they
    /// carry both class fields and methods), so this is a backend-native rendering
    /// entry point rather than a trait method.
    pub fn render_class_to<S: AsRef<str>>(
        &self,
        input: &JsClass,
        before: Option<S>,
        after: Option<S>,
        options: Option<&JsClassRenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = JsClassRenderOptions::default();
        let options: &JsClassRenderOptions = options.unwrap_or(&binding);
        let nl = self.new_line.as_str();

        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }

        if options.render_jsdoc
            && let Some(doc) = &input.doc
        {
            let pad = self.indent(*indent_level);
            write!(out, "{pad}/**{nl}")?;
            for line in doc.split('\n') {
                write!(out, "{pad} * {line}{nl}")?;
            }
            write!(out, "{pad} */{nl}")?;
        }

        write!(out, "{}class {}", self.indent(*indent_level), input.name)?;
        if let Some(extends) = &input.extends {
            write!(out, " extends {extends}")?;
        }
        write!(out, " {{{nl}")?;

        *indent_level += 1;
        for field in &input.fields {
            write!(out, "{}", self.indent(*indent_level))?;
            if field.is_static {
                write!(out, "static ")?;
            }
            write!(out, "{} = {};{nl}", field.name, field.value)?;
        }

        let method_options = JsFunctionRenderOptions {
            render_jsdoc: options.render_jsdoc,
            typescript: options.typescript,
        };
        for (index, method) in input.methods.iter().enumerate() {
            if !input.fields.is_empty() || index > 0 {
                write!(out, "{nl}")?;
            }
            self.write_function(method, &method_options, true, *indent_level, out)?;
        }
        *indent_level -= 1;

        write!(out, "{}}}{nl}", self.indent(*indent_level))?;

        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }
        Ok(())
    }

    /// Render a `class` to a string.
    pub fn render_class<S: AsRef<str>>(
        &self,
        input: &JsClass,
        before: Option<S>,
        after: Option<S>,
        options: Option<&JsClassRenderOptions>,
        indent_level: &mut i32,
    ) -> Result<String, io::Error> {
        let mut output = Vec::new();
        self.render_class_to(input, before, after, options, indent_level, &mut output)?;
        Ok(String::from_utf8(output).expect("Rendered output is not valid UTF-8"))
    }
}
