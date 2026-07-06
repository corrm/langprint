use std::io::{self, Write};

use super::{
    LuaEnum, LuaEnumRenderOptions, LuaFunction, LuaFunctionRenderOptions, LuaModule,
    LuaModuleRenderOptions,
};
use crate::{
    backends::{BackendFeature, BackendMetadata},
    helper::indent,
    renderers::{EnumRenderer, FunctionRenderer},
    text::{IndentStyle, NewLineStyle},
};

/// Backend that renders the Lua native model as idiomatic Lua source.
///
/// langprint emits form only. Lua has no declaration-only function form (a
/// function always closes with `end`), so the body-slot seam maps as: `body`
/// of `None` renders an empty function body, and `Some(lines)` renders each
/// line verbatim one indent deeper between the signature and `end`. The
/// consumer owns the body content.
#[derive(Debug, Clone)]
pub struct LuaBackend {
    /// The newline style to use.
    pub new_line: NewLineStyle,
    /// The indentation style to use.
    pub indent_style: IndentStyle,
    /// The number of spaces per indentation level (when using spaces).
    pub indent_size: i32,
}

impl Default for LuaBackend {
    fn default() -> Self {
        Self {
            new_line: NewLineStyle::LF,
            indent_style: IndentStyle::Spaces,
            indent_size: 2,
        }
    }
}

impl LuaBackend {
    fn indent(&self, level: i32) -> String {
        indent(level, self.indent_size, self.indent_style)
    }

    fn write_doc(
        &self,
        doc: &str,
        indent_level: i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        for line in doc.split('\n') {
            write!(
                out,
                "{}-- {}{}",
                self.indent(indent_level),
                line,
                self.new_line.as_str()
            )?;
        }
        Ok(())
    }

    /// Render a Lua function at the given indentation level.
    fn write_function(
        &self,
        input: &LuaFunction,
        options: &LuaFunctionRenderOptions,
        indent_level: i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        if options.render_doc
            && let Some(doc) = &input.doc
        {
            self.write_doc(doc, indent_level, out)?;
        }

        write!(
            out,
            "{}function {}({}){}",
            self.indent(indent_level),
            input.name,
            input.parameters.join(", "),
            self.new_line.as_str()
        )?;

        if let Some(lines) = &input.body {
            for line in lines {
                write!(
                    out,
                    "{}{}{}",
                    self.indent(indent_level + 1),
                    line,
                    self.new_line.as_str()
                )?;
            }
        }

        write!(
            out,
            "{}end{}",
            self.indent(indent_level),
            self.new_line.as_str()
        )
    }
}

impl BackendMetadata for LuaBackend {
    fn language_name(&self) -> &'static str {
        "Lua"
    }

    fn supported_features(&self) -> &'static [BackendFeature] {
        &[
            BackendFeature::Function,
            BackendFeature::Namespace,
            BackendFeature::Enum,
        ]
    }
}

impl EnumRenderer for LuaBackend {
    type EnumType = LuaEnum;
    type RenderOptions = LuaEnumRenderOptions;

    fn render_enum_to<S: AsRef<str>>(
        &self,
        input: &Self::EnumType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <LuaBackend as EnumRenderer>::default_options();
        let options: &LuaEnumRenderOptions = options.unwrap_or(&binding);
        let pad = self.indent(*indent_level);
        let nl = self.new_line.as_str();

        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }
        if options.render_doc
            && let Some(doc) = &input.doc
        {
            for line in doc.split('\n') {
                write!(out, "{pad}--- {line}{nl}")?;
            }
        }
        write!(out, "{pad}local {} = {{{nl}", input.name)?;
        for member in &input.members {
            write!(
                out,
                "{}{} = {},{nl}",
                self.indent(*indent_level + 1),
                member.name,
                member.value
            )?;
        }
        write!(out, "{pad}}}{nl}")?;
        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }
        Ok(())
    }
}

impl FunctionRenderer for LuaBackend {
    type FunctionType = LuaFunction;
    type RenderOptions = LuaFunctionRenderOptions;

    fn render_function_to<S: AsRef<str>>(
        &self,
        input: &Self::FunctionType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <LuaBackend as FunctionRenderer>::default_options();
        let options: &LuaFunctionRenderOptions = options.unwrap_or(&binding);

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

impl LuaBackend {
    /// Render a Lua module table (`local M = {}` ... `return M`) to a writer.
    ///
    /// A Lua module is not one of the shared renderer traits' shapes (it carries
    /// both field assignments and functions wrapped in return-table scaffolding),
    /// so this is a backend-native rendering entry point rather than a trait method.
    pub fn render_module_to<S: AsRef<str>>(
        &self,
        input: &LuaModule,
        before: Option<S>,
        after: Option<S>,
        options: Option<&LuaModuleRenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = LuaModuleRenderOptions::default();
        let options: &LuaModuleRenderOptions = options.unwrap_or(&binding);

        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }

        if options.render_doc
            && let Some(doc) = &input.doc
        {
            self.write_doc(doc, *indent_level, out)?;
        }

        write!(
            out,
            "{}local {} = {{}}{}",
            self.indent(*indent_level),
            input.table_name,
            self.new_line.as_str()
        )?;

        for field in &input.fields {
            write!(
                out,
                "{}{} = {}{}",
                self.indent(*indent_level),
                field.name,
                field.value,
                self.new_line.as_str()
            )?;
        }

        let function_options = LuaFunctionRenderOptions::DEFAULT;
        for function in &input.functions {
            write!(out, "{}", self.new_line.as_str())?;
            self.write_function(function, &function_options, *indent_level, out)?;
        }

        write!(out, "{}", self.new_line.as_str())?;
        write!(
            out,
            "{}return {}{}",
            self.indent(*indent_level),
            input.table_name,
            self.new_line.as_str()
        )?;

        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }
        Ok(())
    }

    /// Render a Lua module table to a string.
    pub fn render_module<S: AsRef<str>>(
        &self,
        input: &LuaModule,
        before: Option<S>,
        after: Option<S>,
        options: Option<&LuaModuleRenderOptions>,
        indent_level: &mut i32,
    ) -> Result<String, io::Error> {
        let mut output = Vec::new();
        self.render_module_to(input, before, after, options, indent_level, &mut output)?;
        Ok(String::from_utf8(output).expect("Rendered output is not valid UTF-8"))
    }
}
