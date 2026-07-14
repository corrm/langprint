use std::io::{self, Write};

use super::{
    RustConstant, RustConstantRenderOptions, RustDefinition, RustDefinitionRenderOptions, RustEnum,
    RustEnumRenderOptions, RustEnumVariantRenderOptions, RustEnumVariantValue, RustExternBlock,
    RustField, RustFunction, RustFunctionRenderOptions, RustModule, RustModuleRenderOptions,
    RustStruct, RustStructRenderOptions, RustTrait,
    generic_types::{render_generic_decls, render_generic_uses},
};
use crate::{
    backends::{BackendFeature, BackendMetadata},
    helper::indent,
    renderers::{
        ConstantRenderer, DefinitionRenderer, EnumRenderer, FunctionRenderer, NamespaceRenderer,
        StructRenderer,
    },
    text::{IndentStyle, NewLineStyle},
};

/// Backend that renders the neutral declaration model as idiomatic Rust source.
#[derive(Debug, Clone)]
pub struct RustBackend {
    /// The newline style to use.
    pub new_line: NewLineStyle,
    /// The indentation style to use.
    pub indent_style: IndentStyle,
    /// The number of spaces per indentation level (when using spaces).
    pub indent_size: i32,
}

impl Default for RustBackend {
    fn default() -> Self {
        Self {
            new_line: NewLineStyle::LF,
            indent_style: IndentStyle::Spaces,
            indent_size: 4,
        }
    }
}

impl RustBackend {
    fn indent(&self, level: i32) -> String {
        indent(level, self.indent_size, self.indent_style)
    }

    fn write_docs(
        &self,
        docs: &[String],
        indent_level: i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
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

    fn write_comments(
        &self,
        comments: &[String],
        indent_level: i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        for line in comments {
            write!(
                out,
                "{}// {}{}",
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
                "{}#[{}]{}",
                self.indent(indent_level),
                attribute,
                self.new_line.as_str()
            )?;
        }
        Ok(())
    }

    fn write_derives(
        &self,
        derives: &[String],
        indent_level: i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        if derives.is_empty() {
            return Ok(());
        }
        write!(
            out,
            "{}#[derive({})]{}",
            self.indent(indent_level),
            derives.join(", "),
            self.new_line.as_str()
        )
    }

    /// Render a function/method at the given indentation level (used by both the function
    /// renderer and the struct's `impl` block).
    fn write_function(
        &self,
        input: &RustFunction,
        options: &RustFunctionRenderOptions,
        indent_level: i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        if options.render_docs
            && let Some(docs) = &input.docs
        {
            self.write_docs(docs, indent_level, out)?;
        }
        self.write_comments(&input.comments, indent_level, out)?;
        if options.render_attributes {
            self.write_attributes(&input.return_attributes, indent_level, out)?;
            self.write_attributes(&input.attributes, indent_level, out)?;
        }

        write!(
            out,
            "{}{}",
            self.indent(indent_level),
            input.visibility.prefix()
        )?;
        if input.is_const {
            write!(out, "const ")?;
        }
        if input.is_async {
            write!(out, "async ")?;
        }
        if input.is_unsafe {
            write!(out, "unsafe ")?;
        }
        if let Some(abi) = &input.abi {
            write!(out, "extern \"{}\" ", abi)?;
        }
        write!(
            out,
            "fn {}{}(",
            input.name,
            render_generic_decls(&input.generic_args)
        )?;

        let mut first = true;
        if let Some(receiver) = input.self_kind.render() {
            write!(out, "{}", receiver)?;
            first = false;
        }
        for param in &input.parameters {
            if !first {
                write!(out, ", ")?;
            }
            for attribute in &param.attributes {
                write!(out, "#[{}] ", attribute)?;
            }
            write!(out, "{}: {}", param.name, param.param_type)?;
            first = false;
        }
        write!(out, ")")?;

        if let Some(return_type) = &input.return_type
            && !return_type.is_empty()
        {
            write!(out, " -> {}", return_type)?;
        }

        match &input.body {
            Some(lines) => {
                write!(out, " {{{}", self.new_line.as_str())?;
                for line in lines {
                    write!(
                        out,
                        "{}{}{}",
                        self.indent(indent_level + 1),
                        line,
                        self.new_line.as_str()
                    )?;
                }
                write!(
                    out,
                    "{}}}{}",
                    self.indent(indent_level),
                    self.new_line.as_str()
                )?;
            }
            None => write!(out, ";{}", self.new_line.as_str())?,
        }

        Ok(())
    }
}

impl RustBackend {
    /// Render a `trait` declaration to a writer. Each method is a bodyless [`RustFunction`]
    /// (`body: None`), so it renders as a `fn …;` signature. `options` gates doc/attribute
    /// rendering for the trait and its methods.
    pub fn render_trait_to(
        &self,
        input: &RustTrait,
        options: Option<&RustFunctionRenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <RustBackend as FunctionRenderer>::default_options();
        let options: &RustFunctionRenderOptions = options.unwrap_or(&binding);

        if options.render_docs
            && let Some(docs) = &input.docs
        {
            self.write_docs(docs, *indent_level, out)?;
        }
        if options.render_attributes {
            self.write_attributes(&input.attributes, *indent_level, out)?;
        }

        write!(
            out,
            "{}{}trait {}{}",
            self.indent(*indent_level),
            input.visibility.prefix(),
            input.name,
            render_generic_decls(&input.generic_args)
        )?;
        if !input.supertraits.is_empty() {
            write!(out, ": {}", input.supertraits.join(" + "))?;
        }

        if input.methods.is_empty() {
            write!(out, " {{}}{}", self.new_line.as_str())?;
            return Ok(());
        }
        write!(out, " {{{}", self.new_line.as_str())?;
        *indent_level += 1;
        for method in &input.methods {
            self.write_function(method, options, *indent_level, out)?;
        }
        *indent_level -= 1;
        write!(
            out,
            "{}}}{}",
            self.indent(*indent_level),
            self.new_line.as_str()
        )?;
        Ok(())
    }

    /// Render a `trait` declaration to a string.
    pub fn render_trait(
        &self,
        input: &RustTrait,
        options: Option<&RustFunctionRenderOptions>,
        indent_level: &mut i32,
    ) -> Result<String, io::Error> {
        let mut out = Vec::new();
        self.render_trait_to(input, options, indent_level, &mut out)?;
        Ok(String::from_utf8(out).expect("Rendered output is not valid UTF-8"))
    }

    /// Render an `extern` block to a writer. Each item is a bodyless [`RustFunction`]; the block
    /// owns the ABI, so items carry no `abi` of their own.
    pub fn render_extern_block_to(
        &self,
        input: &RustExternBlock,
        options: Option<&RustFunctionRenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <RustBackend as FunctionRenderer>::default_options();
        let options: &RustFunctionRenderOptions = options.unwrap_or(&binding);

        if options.render_docs
            && let Some(docs) = &input.docs
        {
            self.write_docs(docs, *indent_level, out)?;
        }

        write!(out, "{}", self.indent(*indent_level))?;
        if input.is_unsafe {
            write!(out, "unsafe ")?;
        }
        write!(out, "extern \"{}\" {{{}", input.abi, self.new_line.as_str())?;
        *indent_level += 1;
        for item in &input.items {
            self.write_function(item, options, *indent_level, out)?;
        }
        *indent_level -= 1;
        write!(
            out,
            "{}}}{}",
            self.indent(*indent_level),
            self.new_line.as_str()
        )?;
        Ok(())
    }

    /// Render an `extern` block to a string.
    pub fn render_extern_block(
        &self,
        input: &RustExternBlock,
        options: Option<&RustFunctionRenderOptions>,
        indent_level: &mut i32,
    ) -> Result<String, io::Error> {
        let mut out = Vec::new();
        self.render_extern_block_to(input, options, indent_level, &mut out)?;
        Ok(String::from_utf8(out).expect("Rendered output is not valid UTF-8"))
    }
}

impl BackendMetadata for RustBackend {
    fn language_name(&self) -> &'static str {
        "Rust"
    }

    fn supported_features(&self) -> &'static [BackendFeature] {
        &[
            BackendFeature::Constant,
            BackendFeature::Function,
            BackendFeature::Enum,
            BackendFeature::Struct,
        ]
    }
}

impl DefinitionRenderer for RustBackend {
    type DefineType = RustDefinition;
    type RenderOptions = RustDefinitionRenderOptions;

    fn render_definition_to<S: AsRef<str>>(
        &self,
        input: &Self::DefineType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <RustBackend as DefinitionRenderer>::default_options();
        let options: &RustDefinitionRenderOptions = options.unwrap_or(&binding);

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
                "{}pub const {}: {} = {};{}",
                self.indent(*indent_level),
                input.name,
                options.const_type,
                value,
                self.new_line.as_str()
            )?,
            None => write!(
                out,
                "{}pub const {}: () = ();{}",
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

impl ConstantRenderer for RustBackend {
    type ConstantType = RustConstant;
    type RenderOptions = RustConstantRenderOptions;

    fn render_constant_to<S: AsRef<str>>(
        &self,
        input: &Self::ConstantType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <RustBackend as ConstantRenderer>::default_options();
        let options: &RustConstantRenderOptions = options.unwrap_or(&binding);

        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }
        if options.render_docs
            && let Some(docs) = &input.docs
        {
            self.write_docs(docs, *indent_level, out)?;
        }

        let keyword = if input.is_static { "static" } else { "const" };
        write!(
            out,
            "{}{}{} {}: {} = {};{}",
            self.indent(*indent_level),
            input.visibility.prefix(),
            keyword,
            input.name,
            input.data_type,
            input.value,
            self.new_line.as_str()
        )?;

        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }
        Ok(())
    }
}

impl EnumRenderer for RustBackend {
    type EnumType = RustEnum;
    type RenderOptions = RustEnumRenderOptions;

    fn render_enum_to<S: AsRef<str>>(
        &self,
        input: &Self::EnumType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <RustBackend as EnumRenderer>::default_options();
        let options: &RustEnumRenderOptions = options.unwrap_or(&binding);
        let variant_options: &RustEnumVariantRenderOptions = &options.variant;

        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }
        if options.render_docs
            && let Some(docs) = &input.docs
        {
            self.write_docs(docs, *indent_level, out)?;
        }
        if options.render_attributes {
            let write_repr = |out: &mut dyn Write| -> Result<(), io::Error> {
                if let Some(repr) = &input.repr {
                    write!(
                        out,
                        "{}#[repr({})]{}",
                        self.indent(*indent_level),
                        repr,
                        self.new_line.as_str()
                    )?;
                }
                Ok(())
            };
            if options.attributes_before_derives {
                write_repr(out)?;
                self.write_derives(&input.derives, *indent_level, out)?;
            } else {
                self.write_derives(&input.derives, *indent_level, out)?;
                write_repr(out)?;
            }
        }

        write!(
            out,
            "{}{}enum {} {{{}",
            self.indent(*indent_level),
            input.visibility.prefix(),
            input.name,
            self.new_line.as_str()
        )?;

        *indent_level += 1;
        for variant in &input.variants {
            if variant_options.render_docs
                && let Some(docs) = &variant.docs
            {
                self.write_docs(docs, *indent_level, out)?;
            }
            write!(out, "{}{}", self.indent(*indent_level), variant.name)?;
            match &variant.value {
                RustEnumVariantValue::Unit => {}
                RustEnumVariantValue::Discriminant(value) => write!(out, " = {}", value)?,
                RustEnumVariantValue::Tuple(types) => write!(out, "({})", types.join(", "))?,
                RustEnumVariantValue::Struct(fields) => {
                    let rendered = fields
                        .iter()
                        .map(|(name, ty)| format!("{}: {}", name, ty))
                        .collect::<Vec<_>>()
                        .join(", ");
                    write!(out, " {{ {} }}", rendered)?;
                }
            }
            write!(out, ",{}", self.new_line.as_str())?;
        }
        *indent_level -= 1;

        write!(
            out,
            "{}}}{}",
            self.indent(*indent_level),
            self.new_line.as_str()
        )?;

        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }
        Ok(())
    }
}

impl FunctionRenderer for RustBackend {
    type FunctionType = RustFunction;
    type RenderOptions = RustFunctionRenderOptions;

    fn render_function_to<S: AsRef<str>>(
        &self,
        input: &Self::FunctionType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <RustBackend as FunctionRenderer>::default_options();
        let options: &RustFunctionRenderOptions = options.unwrap_or(&binding);

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

impl NamespaceRenderer for RustBackend {
    type NamespaceType = RustModule;
    type RenderOptions = RustModuleRenderOptions;

    fn render_namespace_to<S: AsRef<str>>(
        &self,
        input: &Self::NamespaceType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <RustBackend as NamespaceRenderer>::default_options();
        let options: &RustModuleRenderOptions = options.unwrap_or(&binding);

        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }
        if let Some(docs) = &input.docs {
            self.write_docs(docs, *indent_level, out)?;
        }

        write!(
            out,
            "{}{}mod {} {{{}",
            self.indent(*indent_level),
            input.visibility.prefix(),
            input.name,
            self.new_line.as_str()
        )?;

        *indent_level += 1;
        let mut body_level: i32 = *indent_level;
        let mut blocks: Vec<String> = Vec::new();

        if let Some(defines) = &input.defines {
            for define in defines {
                blocks.push(self.render_definition(
                    define,
                    None::<&str>,
                    None::<&str>,
                    Some(&options.define_options),
                    &mut body_level,
                )?);
            }
        }
        if let Some(constants) = &input.constants {
            for constant in constants {
                blocks.push(self.render_constant(
                    constant,
                    None::<&str>,
                    None::<&str>,
                    Some(&options.constant_options),
                    &mut body_level,
                )?);
            }
        }
        if let Some(enums) = &input.enums {
            for enum_ in enums {
                blocks.push(self.render_enum(
                    enum_,
                    None::<&str>,
                    None::<&str>,
                    Some(&options.enum_options),
                    &mut body_level,
                )?);
            }
        }
        if let Some(structs) = &input.structs {
            for struct_ in structs {
                blocks.push(self.render_struct(
                    struct_,
                    None::<&str>,
                    None::<&str>,
                    Some(&options.struct_options),
                    &mut body_level,
                )?);
            }
        }
        if let Some(functions) = &input.functions {
            for function in functions {
                blocks.push(self.render_function(
                    function,
                    None::<&str>,
                    None::<&str>,
                    Some(&options.function_options),
                    &mut body_level,
                )?);
            }
        }
        if let Some(modules) = &input.modules {
            for module in modules {
                blocks.push(self.render_namespace(
                    module,
                    None::<&str>,
                    None::<&str>,
                    Some(options),
                    &mut body_level,
                )?);
            }
        }
        *indent_level -= 1;

        let separator = format!("{}{}", self.new_line.as_str(), self.new_line.as_str());
        let body = blocks
            .iter()
            .map(|block| block.trim_end_matches(self.new_line.as_str()))
            .collect::<Vec<&str>>()
            .join(&separator);
        if !body.is_empty() {
            write!(out, "{}{}", body, self.new_line.as_str())?;
        }

        write!(
            out,
            "{}}}{}",
            self.indent(*indent_level),
            self.new_line.as_str()
        )?;

        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }

        Ok(())
    }
}

impl StructRenderer for RustBackend {
    type StructType = RustStruct;
    type RenderOptions = RustStructRenderOptions;

    fn render_struct_to<S: AsRef<str>>(
        &self,
        input: &Self::StructType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <RustBackend as StructRenderer>::default_options();
        let options: &RustStructRenderOptions = options.unwrap_or(&binding);

        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }
        if options.render_docs
            && let Some(docs) = &input.docs
        {
            self.write_docs(docs, *indent_level, out)?;
        }
        if options.render_attributes {
            if options.attributes_before_derives {
                self.write_attributes(&input.attributes, *indent_level, out)?;
                self.write_derives(&input.derives, *indent_level, out)?;
            } else {
                self.write_derives(&input.derives, *indent_level, out)?;
                self.write_attributes(&input.attributes, *indent_level, out)?;
            }
        }

        let generics = render_generic_decls(&input.generic_args);
        write!(
            out,
            "{}{}struct {}{}",
            self.indent(*indent_level),
            input.visibility.prefix(),
            input.name,
            generics
        )?;

        if input.is_tuple {
            let rendered = input
                .fields
                .iter()
                .map(|field| format!("{}{}", field.visibility.prefix(), field.field_type))
                .collect::<Vec<_>>()
                .join(", ");
            write!(out, "({});{}", rendered, self.new_line.as_str())?;
        } else {
            write!(out, " {{{}", self.new_line.as_str())?;
            *indent_level += 1;
            for field in &input.fields {
                self.write_field(field, options, *indent_level, out)?;
            }
            *indent_level -= 1;
            write!(
                out,
                "{}}}{}",
                self.indent(*indent_level),
                self.new_line.as_str()
            )?;
        }

        if options.render_impl && !input.methods.is_empty() {
            let fn_options = RustFunctionRenderOptions::DEFAULT;
            write!(
                out,
                "{}{}impl{} {}{} {{{}",
                self.new_line.as_str(),
                self.indent(*indent_level),
                generics,
                input.name,
                render_generic_uses(&input.generic_args),
                self.new_line.as_str()
            )?;
            *indent_level += 1;
            for (i, method) in input.methods.iter().enumerate() {
                if i > 0 {
                    write!(out, "{}", self.new_line.as_str())?;
                }
                self.write_function(method, &fn_options, *indent_level, out)?;
            }
            *indent_level -= 1;
            write!(
                out,
                "{}}}{}",
                self.indent(*indent_level),
                self.new_line.as_str()
            )?;
        }

        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }
        Ok(())
    }
}

impl RustBackend {
    fn write_field(
        &self,
        field: &RustField,
        options: &RustStructRenderOptions,
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
        write!(
            out,
            "{}{}{}: {},{}",
            self.indent(indent_level),
            field.visibility.prefix(),
            field.name,
            field.field_type,
            self.new_line.as_str()
        )
    }
}
