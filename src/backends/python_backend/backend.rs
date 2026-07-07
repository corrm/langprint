use std::io::{self, Write};

use super::{
    PythonClass, PythonClassRenderOptions, PythonEnum, PythonEnumRenderOptions, PythonFunction,
    PythonFunctionRenderOptions, PythonStruct, PythonStructRenderOptions,
};
use crate::{
    backends::{BackendFeature, BackendMetadata},
    helper::indent,
    renderers::{EnumRenderer, FunctionRenderer, StructRenderer},
    text::{IndentStyle, NewLineStyle},
};

/// Backend that renders the neutral declaration model as idiomatic Python source.
///
/// langprint emits form only: every `def` whose `body` is `None` renders a single
/// `pass` (Python's declaration form, since there is no empty block), and a `body`
/// of `Some(lines)` renders each line verbatim one level deeper. The consumer owns
/// the body content.
#[derive(Debug, Clone)]
pub struct PythonBackend {
    /// The newline style to use.
    pub new_line: NewLineStyle,
    /// The indentation style to use.
    pub indent_style: IndentStyle,
    /// The number of spaces per indentation level (when using spaces).
    pub indent_size: i32,
    /// When `true`, a multi-line docstring's closing `"""` is written on its own
    /// indented line (PEP 257 style) instead of being appended to the last
    /// content line. Single-line docstrings are unaffected. Defaults to `false`
    /// to preserve the compact appended form.
    pub docstring_close_on_own_line: bool,
    /// When `true`, a docstring whose content contains a backslash is rendered as
    /// a raw string (`r"""…"""`), so an embedded escape sequence does not raise a
    /// `SyntaxWarning` on import. Defaults to `false`.
    pub docstring_raw_on_backslash: bool,
}

impl Default for PythonBackend {
    fn default() -> Self {
        Self {
            new_line: NewLineStyle::LF,
            indent_style: IndentStyle::Spaces,
            indent_size: 4,
            docstring_close_on_own_line: false,
            docstring_raw_on_backslash: false,
        }
    }
}

impl PythonBackend {
    fn indent(&self, level: i32) -> String {
        indent(level, self.indent_size, self.indent_style)
    }

    fn write_docstring(
        &self,
        docstring: &str,
        indent_level: i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let prefix: &str = if self.docstring_raw_on_backslash && docstring.contains('\\') {
            "r"
        } else {
            ""
        };
        let mut multiline = false;
        for (index, line) in docstring.split('\n').enumerate() {
            if index == 0 {
                write!(out, "{}{}\"\"\"{}", self.indent(indent_level), prefix, line)?;
            } else {
                multiline = true;
                write!(
                    out,
                    "{}{}{}",
                    self.new_line.as_str(),
                    self.indent(indent_level),
                    line
                )?;
            }
        }
        if multiline && self.docstring_close_on_own_line {
            write!(
                out,
                "{}{}\"\"\"{}",
                self.new_line.as_str(),
                self.indent(indent_level),
                self.new_line.as_str()
            )
        } else {
            write!(out, "\"\"\"{}", self.new_line.as_str())
        }
    }

    /// Render a `def` (function or method) at the given indentation level.
    fn write_function(
        &self,
        input: &PythonFunction,
        options: &PythonFunctionRenderOptions,
        indent_level: i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        write!(out, "{}def {}(", self.indent(indent_level), input.name)?;

        let mut first = true;
        for param in &input.parameters {
            if !first {
                write!(out, ", ")?;
            }
            write!(out, "{}", param.name)?;
            if let Some(type_hint) = &param.type_hint {
                write!(out, ": {}", type_hint)?;
            }
            if let Some(default) = &param.default {
                if param.type_hint.is_some() {
                    write!(out, " = {}", default)?;
                } else {
                    write!(out, "={}", default)?;
                }
            }
            first = false;
        }
        write!(out, ")")?;

        if let Some(return_type) = &input.return_type {
            write!(out, " -> {}", return_type)?;
        }
        write!(out, ":{}", self.new_line.as_str())?;

        let body_level = indent_level + 1;
        let mut wrote_body = false;

        if options.render_docstring
            && let Some(docstring) = &input.docstring
        {
            self.write_docstring(docstring, body_level, out)?;
            wrote_body = true;
        }

        if let Some(lines) = &input.body {
            let body_indent: String = if options.verbatim_body {
                String::new()
            } else {
                self.indent(body_level)
            };
            for line in lines {
                write!(out, "{}{}{}", body_indent, line, self.new_line.as_str())?;
                wrote_body = true;
            }
        }

        if !wrote_body {
            write!(
                out,
                "{}pass{}",
                self.indent(body_level),
                self.new_line.as_str()
            )?;
        }

        Ok(())
    }
}

impl BackendMetadata for PythonBackend {
    fn language_name(&self) -> &'static str {
        "Python"
    }

    fn supported_features(&self) -> &'static [BackendFeature] {
        &[
            BackendFeature::Function,
            BackendFeature::Enum,
            BackendFeature::Struct,
            BackendFeature::Class,
        ]
    }
}

impl FunctionRenderer for PythonBackend {
    type FunctionType = PythonFunction;
    type RenderOptions = PythonFunctionRenderOptions;

    fn render_function_to<S: AsRef<str>>(
        &self,
        input: &Self::FunctionType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <PythonBackend as FunctionRenderer>::default_options();
        let options: &PythonFunctionRenderOptions = options.unwrap_or(&binding);

        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }
        self.write_function(input, options, *indent_level, out)?;
        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }
        Ok(())
    }
}

impl StructRenderer for PythonBackend {
    type StructType = PythonStruct;
    type RenderOptions = PythonStructRenderOptions;

    fn render_struct_to<S: AsRef<str>>(
        &self,
        input: &Self::StructType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <PythonBackend as StructRenderer>::default_options();
        let options: &PythonStructRenderOptions = options.unwrap_or(&binding);

        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }

        write!(
            out,
            "{}class {}({}):{}",
            self.indent(*indent_level),
            input.name,
            input.base_class,
            self.new_line.as_str()
        )?;

        *indent_level += 1;
        if options.render_docstring
            && let Some(docstring) = &input.docstring
        {
            self.write_docstring(docstring, *indent_level, out)?;
        }

        write!(
            out,
            "{}_fields_ = [{}",
            self.indent(*indent_level),
            self.new_line.as_str()
        )?;
        *indent_level += 1;
        for field in &input.fields {
            write!(
                out,
                "{}(\"{}\", {}),{}",
                self.indent(*indent_level),
                field.name,
                field.ctype,
                self.new_line.as_str()
            )?;
        }
        *indent_level -= 1;
        write!(
            out,
            "{}]{}",
            self.indent(*indent_level),
            self.new_line.as_str()
        )?;
        *indent_level -= 1;

        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }
        Ok(())
    }
}

impl EnumRenderer for PythonBackend {
    type EnumType = PythonEnum;
    type RenderOptions = PythonEnumRenderOptions;

    fn render_enum_to<S: AsRef<str>>(
        &self,
        input: &Self::EnumType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <PythonBackend as EnumRenderer>::default_options();
        let options: &PythonEnumRenderOptions = options.unwrap_or(&binding);

        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }

        write!(
            out,
            "{}class {}({}):{}",
            self.indent(*indent_level),
            input.name,
            input.base_class,
            self.new_line.as_str()
        )?;

        *indent_level += 1;
        if options.render_docstring
            && let Some(docstring) = &input.docstring
        {
            self.write_docstring(docstring, *indent_level, out)?;
        }

        if input.members.is_empty() {
            write!(
                out,
                "{}pass{}",
                self.indent(*indent_level),
                self.new_line.as_str()
            )?;
        } else {
            for member in &input.members {
                write!(
                    out,
                    "{}{} = {}{}",
                    self.indent(*indent_level),
                    member.name,
                    member.value,
                    self.new_line.as_str()
                )?;
            }
        }
        *indent_level -= 1;

        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }
        Ok(())
    }
}

impl PythonBackend {
    /// Render a plain `class` (`class Name:` / `class Name(Base):`) to a writer.
    ///
    /// Python plain classes are not one of the shared renderer traits' shapes
    /// (they carry both class-level field assignments and `def` methods), so this
    /// is a backend-native rendering entry point rather than a trait method.
    pub fn render_class_to<S: AsRef<str>>(
        &self,
        input: &PythonClass,
        before: Option<S>,
        after: Option<S>,
        options: Option<&PythonClassRenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = PythonClassRenderOptions::default();
        let options: &PythonClassRenderOptions = options.unwrap_or(&binding);

        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }

        write!(out, "{}class {}", self.indent(*indent_level), input.name)?;
        if !input.bases.is_empty() {
            write!(out, "({})", input.bases.join(", "))?;
        }
        write!(out, ":{}", self.new_line.as_str())?;

        *indent_level += 1;
        let mut wrote_body = false;

        if options.render_docstring
            && let Some(docstring) = &input.docstring
        {
            self.write_docstring(docstring, *indent_level, out)?;
            wrote_body = true;
        }

        for field in &input.fields {
            write!(
                out,
                "{}{} = {}{}",
                self.indent(*indent_level),
                field.name,
                field.value,
                self.new_line.as_str()
            )?;
            wrote_body = true;
        }

        let method_options = PythonFunctionRenderOptions::DEFAULT;
        for (index, method) in input.methods.iter().enumerate() {
            if wrote_body || index > 0 {
                write!(out, "{}", self.new_line.as_str())?;
            }
            self.write_function(method, &method_options, *indent_level, out)?;
            wrote_body = true;
        }

        if !wrote_body {
            write!(
                out,
                "{}pass{}",
                self.indent(*indent_level),
                self.new_line.as_str()
            )?;
        }
        *indent_level -= 1;

        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }
        Ok(())
    }

    /// Render a plain `class` to a string.
    pub fn render_class<S: AsRef<str>>(
        &self,
        input: &PythonClass,
        before: Option<S>,
        after: Option<S>,
        options: Option<&PythonClassRenderOptions>,
        indent_level: &mut i32,
    ) -> Result<String, io::Error> {
        let mut output = Vec::new();
        self.render_class_to(input, before, after, options, indent_level, &mut output)?;
        Ok(String::from_utf8(output).expect("Rendered output is not valid UTF-8"))
    }
}
