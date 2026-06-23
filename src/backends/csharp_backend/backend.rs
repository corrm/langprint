use std::io::{self, Write};

use super::{
    CSharpConstant, CSharpConstantRenderOptions, CSharpDefinition, CSharpDefinitionRenderOptions, CSharpEnum,
    CSharpEnumRenderOptions, CSharpEnumVariantRenderOptions, CSharpField, CSharpFieldRenderOptions, CSharpMethod,
    CSharpMethodRenderOptions, CSharpProperty, CSharpType, CSharpTypeRenderOptions,
    generic_types::{render_generic_decls, render_where_clauses},
};
use crate::{
    backends::{BackendFeature, BackendMetadata},
    helper::indent,
    renderers::{ConstantRenderer, DefinitionRenderer, EnumRenderer, FunctionRenderer, StructRenderer},
    text::{IndentStyle, NewLineStyle},
};

/// Backend that renders the neutral declaration model as idiomatic C# source.
#[derive(Debug, Clone)]
pub struct CSharpBackend {
    /// The newline style to use.
    pub new_line: NewLineStyle,
    /// The indentation style to use.
    pub indent_style: IndentStyle,
    /// The number of spaces per indentation level (when using spaces).
    pub indent_size: i32,
}

impl Default for CSharpBackend {
    fn default() -> Self {
        Self {
            new_line: NewLineStyle::LF,
            indent_style: IndentStyle::Spaces,
            indent_size: 4,
        }
    }
}

impl CSharpBackend {
    fn indent(&self, level: i32) -> String {
        indent(level, self.indent_size, self.indent_style)
    }

    fn write_docs(&self, docs: &[String], indent_level: i32, out: &mut impl Write) -> Result<(), io::Error> {
        for line in docs {
            write!(
                out,
                "{}/// {}{}",
                self.indent(indent_level),
                line,
                self.new_line.as_str()
            )?;
        }
        Ok(())
    }

    fn write_attributes(
        &self,
        attributes: &[String],
        indent_level: i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        for attribute in attributes {
            write!(
                out,
                "{}[{}]{}",
                self.indent(indent_level),
                attribute,
                self.new_line.as_str()
            )?;
        }
        Ok(())
    }

    fn write_field(
        &self,
        field: &CSharpField,
        options: &CSharpFieldRenderOptions,
        indent_level: i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        if options.render_docs
            && let Some(docs) = &field.docs
        {
            self.write_docs(docs, indent_level, out)?;
        }
        if options.render_attributes {
            self.write_attributes(&field.attributes, indent_level, out)?;
        }

        write!(out, "{}{}", self.indent(indent_level), field.visibility.prefix())?;
        if field.is_static {
            write!(out, "static ")?;
        }
        if field.is_const {
            write!(out, "const ")?;
        }
        if field.is_readonly {
            write!(out, "readonly ")?;
        }
        write!(out, "{} {}", field.field_type, field.name)?;
        if let Some(initializer) = &field.initializer {
            write!(out, " = {}", initializer)?;
        }
        write!(out, ";{}", self.new_line.as_str())
    }

    fn write_property(
        &self,
        property: &CSharpProperty,
        indent_level: i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        if let Some(docs) = &property.docs {
            self.write_docs(docs, indent_level, out)?;
        }

        write!(out, "{}{}", self.indent(indent_level), property.visibility.prefix())?;
        if property.is_static {
            write!(out, "static ")?;
        }
        write!(out, "{} {}", property.prop_type, property.name)?;

        let auto = property.getter_body.is_none() && property.setter_body.is_none();
        if auto {
            write!(out, " {{")?;
            if property.has_getter {
                write!(out, " get;")?;
            }
            if property.has_setter {
                write!(out, " set;")?;
            }
            write!(out, " }}{}", self.new_line.as_str())?;
            return Ok(());
        }

        write!(
            out,
            "{}{}{{{}",
            self.new_line.as_str(),
            self.indent(indent_level),
            self.new_line.as_str()
        )?;
        if property.has_getter {
            self.write_accessor("get", property.getter_body.as_deref(), indent_level + 1, out)?;
        }
        if property.has_setter {
            self.write_accessor("set", property.setter_body.as_deref(), indent_level + 1, out)?;
        }
        write!(out, "{}}}{}", self.indent(indent_level), self.new_line.as_str())
    }

    fn write_accessor(
        &self,
        keyword: &str,
        body: Option<&[String]>,
        indent_level: i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        match body {
            None => write!(
                out,
                "{}{};{}",
                self.indent(indent_level),
                keyword,
                self.new_line.as_str()
            ),
            Some(lines) => {
                write!(
                    out,
                    "{}{}{}",
                    self.indent(indent_level),
                    keyword,
                    self.new_line.as_str()
                )?;
                write!(out, "{}{{{}", self.indent(indent_level), self.new_line.as_str())?;
                for line in lines {
                    write!(
                        out,
                        "{}{}{}",
                        self.indent(indent_level + 1),
                        line,
                        self.new_line.as_str()
                    )?;
                }
                write!(out, "{}}}{}", self.indent(indent_level), self.new_line.as_str())
            }
        }
    }

    fn write_method(
        &self,
        method: &CSharpMethod,
        options: &CSharpMethodRenderOptions,
        indent_level: i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        if options.render_docs
            && let Some(docs) = &method.docs
        {
            self.write_docs(docs, indent_level, out)?;
        }
        if options.render_attributes {
            self.write_attributes(&method.attributes, indent_level, out)?;
        }

        write!(out, "{}{}", self.indent(indent_level), method.visibility.prefix())?;
        if method.is_static {
            write!(out, "static ")?;
        }
        if method.is_abstract {
            write!(out, "abstract ")?;
        }
        if method.is_virtual {
            write!(out, "virtual ")?;
        }
        if method.is_override {
            write!(out, "override ")?;
        }
        if method.is_sealed && method.is_override {
            write!(out, "sealed ")?;
        }
        if method.is_async {
            write!(out, "async ")?;
        }

        let return_type = method
            .return_type
            .as_deref()
            .unwrap_or(if method.is_async { "Task" } else { "void" });
        write!(
            out,
            "{} {}{}(",
            return_type,
            method.name,
            render_generic_decls(&method.generic_args)
        )?;
        let params = method
            .parameters
            .iter()
            .map(|param| match &param.default_value {
                Some(default) => format!("{} {} = {}", param.param_type, param.name, default),
                None => format!("{} {}", param.param_type, param.name),
            })
            .collect::<Vec<_>>()
            .join(", ");
        write!(out, "{})", params)?;
        write!(out, "{}", render_where_clauses(&method.generic_args))?;

        match &method.body {
            None => write!(out, ";{}", self.new_line.as_str()),
            Some(lines) => {
                write!(
                    out,
                    "{}{}{{{}",
                    self.new_line.as_str(),
                    self.indent(indent_level),
                    self.new_line.as_str()
                )?;
                for line in lines {
                    write!(
                        out,
                        "{}{}{}",
                        self.indent(indent_level + 1),
                        line,
                        self.new_line.as_str()
                    )?;
                }
                write!(out, "{}}}{}", self.indent(indent_level), self.new_line.as_str())
            }
        }
    }
}

impl BackendMetadata for CSharpBackend {
    fn language_name(&self) -> &'static str {
        "C#"
    }

    fn supported_features(&self) -> &'static [BackendFeature] {
        &[
            BackendFeature::Constant,
            BackendFeature::Function,
            BackendFeature::Enum,
            BackendFeature::Struct,
            BackendFeature::Class,
            BackendFeature::Interface,
        ]
    }
}

impl DefinitionRenderer for CSharpBackend {
    type DefineType = CSharpDefinition;
    type RenderOptions = CSharpDefinitionRenderOptions;

    fn render_definition_to<S: AsRef<str>>(
        &self,
        input: &Self::DefineType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <CSharpBackend as DefinitionRenderer>::DEFAULT_RENDER_OPTIONS;
        let options: &CSharpDefinitionRenderOptions = options.unwrap_or(&binding);

        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }
        if options.render_docs
            && let Some(docs) = &input.docs
        {
            self.write_docs(docs, *indent_level, out)?;
        }
        match &input.value {
            Some(value) => write!(
                out,
                "{}public const {} {} = {};{}",
                self.indent(*indent_level),
                super::define_types::infer_const_type(value).unwrap_or(options.const_type),
                input.name,
                value,
                self.new_line.as_str()
            )?,
            None => write!(
                out,
                "{}public const bool {} = true;{}",
                self.indent(*indent_level),
                input.name,
                self.new_line.as_str()
            )?,
        }
        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }
        Ok(())
    }
}

impl ConstantRenderer for CSharpBackend {
    type ConstantType = CSharpConstant;
    type RenderOptions = CSharpConstantRenderOptions;

    fn render_constant_to<S: AsRef<str>>(
        &self,
        input: &Self::ConstantType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <CSharpBackend as ConstantRenderer>::DEFAULT_RENDER_OPTIONS;
        let options: &CSharpConstantRenderOptions = options.unwrap_or(&binding);

        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }
        if options.render_docs
            && let Some(docs) = &input.docs
        {
            self.write_docs(docs, *indent_level, out)?;
        }
        write!(
            out,
            "{}{}const {} {} = {};{}",
            self.indent(*indent_level),
            input.visibility.prefix(),
            input.data_type,
            input.name,
            input.value,
            self.new_line.as_str()
        )?;
        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }
        Ok(())
    }
}

impl EnumRenderer for CSharpBackend {
    type EnumType = CSharpEnum;
    type EnumVariantRenderOptions = CSharpEnumVariantRenderOptions;
    type RenderOptions = CSharpEnumRenderOptions;

    fn render_enum_to<S: AsRef<str>>(
        &self,
        input: &Self::EnumType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        variant_options: Option<&Self::EnumVariantRenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <CSharpBackend as EnumRenderer>::DEFAULT_RENDER_OPTIONS;
        let options: &CSharpEnumRenderOptions = options.unwrap_or(&binding);
        let variant_binding = <CSharpBackend as EnumRenderer>::DEFAULT_ENUM_VARIANT_RENDER_OPTIONS;
        let variant_options: &CSharpEnumVariantRenderOptions = variant_options.unwrap_or(&variant_binding);

        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }
        if options.render_docs
            && let Some(docs) = &input.docs
        {
            self.write_docs(docs, *indent_level, out)?;
        }
        if input.is_flags {
            write!(out, "{}[Flags]{}", self.indent(*indent_level), self.new_line.as_str())?;
        }
        self.write_attributes(&input.attributes, *indent_level, out)?;

        write!(
            out,
            "{}{}enum {}",
            self.indent(*indent_level),
            input.visibility.prefix(),
            input.name
        )?;
        if let Some(underlying) = &input.underlying_type {
            write!(out, " : {}", underlying)?;
        }
        write!(
            out,
            "{}{}{{{}",
            self.new_line.as_str(),
            self.indent(*indent_level),
            self.new_line.as_str()
        )?;

        *indent_level += 1;
        for member in &input.members {
            if variant_options.render_docs
                && let Some(docs) = &member.docs
            {
                self.write_docs(docs, *indent_level, out)?;
            }
            write!(out, "{}{}", self.indent(*indent_level), member.name)?;
            if let Some(value) = &member.value {
                write!(out, " = {}", value)?;
            }
            write!(out, ",{}", self.new_line.as_str())?;
        }
        *indent_level -= 1;

        write!(out, "{}}}{}", self.indent(*indent_level), self.new_line.as_str())?;
        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }
        Ok(())
    }
}

impl FunctionRenderer for CSharpBackend {
    type FunctionType = CSharpMethod;
    type RenderOptions = CSharpMethodRenderOptions;

    fn render_function_to<S: AsRef<str>>(
        &self,
        input: &Self::FunctionType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <CSharpBackend as FunctionRenderer>::DEFAULT_RENDER_OPTIONS;
        let options: &CSharpMethodRenderOptions = options.unwrap_or(&binding);

        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }
        self.write_method(input, options, *indent_level, out)?;
        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }
        Ok(())
    }
}

impl StructRenderer for CSharpBackend {
    type StructType = CSharpType;
    type RenderOptions = CSharpTypeRenderOptions;

    fn render_struct_to<S: AsRef<str>>(
        &self,
        input: &Self::StructType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <CSharpBackend as StructRenderer>::DEFAULT_RENDER_OPTIONS;
        let options: &CSharpTypeRenderOptions = options.unwrap_or(&binding);

        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }
        if options.render_docs
            && let Some(docs) = &input.docs
        {
            self.write_docs(docs, *indent_level, out)?;
        }
        if options.render_attributes {
            self.write_attributes(&input.attributes, *indent_level, out)?;
        }

        write!(out, "{}{}", self.indent(*indent_level), input.visibility.prefix())?;
        if input.is_static {
            write!(out, "static ")?;
        }
        if input.is_abstract && input.kind.can_be_abstract() {
            write!(out, "abstract ")?;
        }
        if input.is_sealed && input.kind.can_be_sealed() {
            write!(out, "sealed ")?;
        }
        if input.is_partial {
            write!(out, "partial ")?;
        }
        write!(
            out,
            "{} {}{}",
            input.kind.keyword(),
            input.name,
            render_generic_decls(&input.generic_args)
        )?;

        let mut bases: Vec<&str> = Vec::new();
        if let Some(base_class) = &input.base_class {
            bases.push(base_class);
        }
        bases.extend(input.interfaces.iter().map(String::as_str));
        if !bases.is_empty() {
            write!(out, " : {}", bases.join(", "))?;
        }
        write!(out, "{}", render_where_clauses(&input.generic_args))?;

        write!(
            out,
            "{}{}{{{}",
            self.new_line.as_str(),
            self.indent(*indent_level),
            self.new_line.as_str()
        )?;

        *indent_level += 1;
        let field_options = CSharpFieldRenderOptions::DEFAULT;
        let method_options = CSharpMethodRenderOptions::DEFAULT;
        let mut wrote_member = false;
        for field in &input.fields {
            self.write_field(field, &field_options, *indent_level, out)?;
            wrote_member = true;
        }
        if options.render_properties {
            for property in &input.properties {
                if wrote_member {
                    write!(out, "{}", self.new_line.as_str())?;
                }
                self.write_property(property, *indent_level, out)?;
                wrote_member = true;
            }
        }
        if options.render_methods {
            for method in &input.methods {
                if wrote_member {
                    write!(out, "{}", self.new_line.as_str())?;
                }
                self.write_method(method, &method_options, *indent_level, out)?;
                wrote_member = true;
            }
        }
        *indent_level -= 1;

        write!(out, "{}}}{}", self.indent(*indent_level), self.new_line.as_str())?;
        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }
        Ok(())
    }
}
