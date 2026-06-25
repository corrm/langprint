use std::io::{self, Write};

use super::{
    CppConstant, CppConstantRenderOptions, CppDefinition, CppDefinitionRenderOptions, CppEnum, CppEnumRenderOptions,
    CppFunction, CppFunctionRenderOptions, CppNamespace, CppNamespaceRenderOptions, CppStruct, CppStructRenderOptions,
    CppVisibility, enum_types::CppEnumVariantRenderOptions,
};
use crate::backends::cpp_backend::struct_types::CppStructKind;
use crate::{
    backends::{BackendFeature, BackendMetadata},
    helper::indent,
    renderers::{ConstantRenderer, DefinitionRenderer, EnumRenderer, FunctionRenderer, NamespaceRenderer, StructRenderer},
    text::{IndentStyle, NewLineStyle},
};

#[derive(Debug, Clone)]
pub enum DocsStyle {
    DoubleSlash,
    TripleSlash,
    SlashAsterisk,
    SlashDoubleAsterisk,
}

/// Backend for C++ type conversion and rendering.
#[derive(Debug, Clone)]
pub struct CppBackend {
    /// The string to use for new lines.
    pub new_line: NewLineStyle,
    /// Whether to add a new line before the open brace.
    pub open_brace_on_new_line: bool,
    /// The style of documentation to use.
    pub docs_style: DocsStyle,
    /// The style of indentation to use.
    pub indent_style: IndentStyle,
    /// The number of spaces to use for indentation.
    pub indent_size: i32,
}

impl BackendMetadata for CppBackend {
    fn language_name(&self) -> &'static str {
        "C++"
    }

    fn supported_features(&self) -> &'static [BackendFeature] {
        &[
            BackendFeature::Define,
            BackendFeature::Namespace,
            BackendFeature::Constant,
            BackendFeature::Function,
            BackendFeature::Enum,
            BackendFeature::Struct,
            BackendFeature::Class,
        ]
    }
}

impl DefinitionRenderer for CppBackend {
    type DefineType = CppDefinition;
    type RenderOptions = CppDefinitionRenderOptions;

    fn render_definition_to<S: AsRef<str>>(
        &self,
        input: &Self::DefineType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <CppBackend as DefinitionRenderer>::default_options();
        let options: &CppDefinitionRenderOptions = options.unwrap_or(&binding);

        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }

        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }

        // Add documentation if available and render_docs is enabled
        if options.render_docs
            && let Some(docs) = &input.docs
        {
            self.write_docs(docs, indent_level, out)?;
        }

        write!(
            out,
            "{}#define {}",
            indent(*indent_level, self.indent_size, self.indent_style),
            input.name,
        )?;

        if let Some(value) = &input.value {
            write!(out, " {}", value)?;
        }

        Ok(())
    }
}

impl ConstantRenderer for CppBackend {
    type ConstantType = CppConstant;
    type RenderOptions = CppConstantRenderOptions;

    fn render_constant_to<S: AsRef<str>>(
        &self,
        input: &Self::ConstantType,
        _before: Option<S>,
        _after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <CppBackend as ConstantRenderer>::default_options();
        let options: &CppConstantRenderOptions = options.unwrap_or(&binding);

        // Add documentation if available and render_docs is enabled
        if options.render_docs
            && let Some(docs) = &input.docs
        {
            self.write_docs(docs, indent_level, out)?;
        }

        // Write constant declaration with visibility
        if input.visibility != CppVisibility::Default {
            write!(out, "{}:{}", input.visibility, self.new_line.as_str())?;
        }

        // Determine which keyword to use based on options
        let keyword: &str = if options.use_constexpr { "constexpr" } else { "const" };

        // Add inline if requested
        let inline_prefix: &str = if options.use_inline { "inline " } else { "" };

        // Write constant declaration
        write!(
            out,
            "{}{}{} {} {} = {};{}",
            indent(*indent_level, self.indent_size, self.indent_style),
            inline_prefix,
            keyword,
            input.data_type,
            input.name,
            input.value,
            self.new_line.as_str(),
        )?;

        Ok(())
    }
}

impl EnumRenderer for CppBackend {
    type EnumType = CppEnum;
    type EnumVariantRenderOptions = CppEnumVariantRenderOptions;
    type RenderOptions = CppEnumRenderOptions;

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
        let binding = <CppBackend as EnumRenderer>::default_options();
        let options: &CppEnumRenderOptions = options.unwrap_or(&binding);

        let variant_binding = <CppBackend as EnumRenderer>::default_variant_options();
        let variant_options: &CppEnumVariantRenderOptions = variant_options.unwrap_or(&variant_binding);

        // Add documentation if available and render_docs is enabled
        if options.render_docs
            && let Some(docs) = &input.docs
        {
            self.write_docs(docs, indent_level, out)?;
        }

        // Write before string if available
        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }

        // Write enum declaration
        if input.is_enum_class {
            write!(
                out,
                "{}enum class {}",
                indent(*indent_level, self.indent_size, self.indent_style),
                input.name
            )?;
        } else {
            write!(
                out,
                "{}enum {}",
                indent(*indent_level, self.indent_size, self.indent_style),
                input.name
            )?;
        }

        // Write underlying type if specified
        if let Some(underlying_type) = &input.underlying_type {
            write!(out, ": {}", underlying_type)?;
        }

        // Write enum body start
        if self.open_brace_on_new_line {
            write!(
                out,
                "{}{}{{{}",
                self.new_line.as_str(),
                indent(*indent_level, self.indent_size, self.indent_style),
                self.new_line.as_str(),
            )?;
        } else {
            write!(out, " {{{}", self.new_line.as_str())?;
        }
        *indent_level += 1;

        // Write enum variants
        let max_variant_name_length: usize = if variant_options.align_value {
            input
                .variants
                .iter()
                .map(|variant| variant.name.len())
                .max()
                .unwrap_or(0)
        } else {
            0
        };

        for variant in &input.variants {
            if variant_options.render_docs
                && let Some(docs) = &variant.docs
            {
                self.write_docs(docs, indent_level, out)?;
            }

            let variant_name: &String = if variant_options.align_value {
                &format!(
                    "{}{}",
                    variant.name,
                    " ".repeat(max_variant_name_length - variant.name.len())
                )
            } else {
                &variant.name
            };

            if let Some(value) = &variant.value {
                write!(
                    out,
                    "{}{} = {},{}",
                    indent(*indent_level, self.indent_size, self.indent_style),
                    variant_name,
                    value,
                    self.new_line.as_str()
                )?;
            } else {
                write!(
                    out,
                    "{}{},{}",
                    indent(*indent_level, self.indent_size, self.indent_style),
                    variant_name,
                    self.new_line.as_str()
                )?;
            }
        }

        // Write enum body end
        *indent_level -= 1;
        write!(
            out,
            "{}}};{}",
            indent(*indent_level, self.indent_size, self.indent_style),
            self.new_line.as_str()
        )?;

        // Write after string if available
        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }

        Ok(())
    }
}

impl NamespaceRenderer for CppBackend {
    type NamespaceType = CppNamespace;
    type RenderOptions = CppNamespaceRenderOptions;

    fn render_namespace_to<S: AsRef<str>>(
        &self,
        input: &Self::NamespaceType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <CppBackend as NamespaceRenderer>::default_options();
        let options: &CppNamespaceRenderOptions = options.unwrap_or(&binding);

        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }

        let indent_str = indent(*indent_level, self.indent_size, self.indent_style);
        write!(out, "{}namespace {}", indent_str, input.name)?;
        if self.open_brace_on_new_line {
            write!(out, "{}{}{{{}", self.new_line.as_str(), indent_str, self.new_line.as_str())?;
        } else {
            write!(out, " {{{}", self.new_line.as_str())?;
        }

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
                    Some(&options.enum_variant_options),
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
        if let Some(namespaces) = &input.namespaces {
            for namespace in namespaces {
                blocks.push(self.render_namespace(namespace, None::<&str>, None::<&str>, Some(options), &mut body_level)?);
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
            indent(*indent_level, self.indent_size, self.indent_style),
            self.new_line.as_str()
        )?;

        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }

        Ok(())
    }
}

impl FunctionRenderer for CppBackend {
    type FunctionType = CppFunction;
    type RenderOptions = CppFunctionRenderOptions;

    fn render_function_to<S: AsRef<str>>(
        &self,
        input: &Self::FunctionType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl io::Write,
    ) -> Result<(), io::Error> {
        let binding = <CppBackend as FunctionRenderer>::default_options();
        let options: &CppFunctionRenderOptions = options.unwrap_or(&binding);

        // Write 'before' content if provided
        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }

        // Write documentation: render docs if rendering AND (docs_on_definition implies render_definition)
        if options.render_docs
            && (!options.docs_on_definition || options.render_definition)
            && let Some(docs) = &input.docs
        {
            self.write_docs(docs, indent_level, out)?;
        }

        // Write indentation
        let indent_str: String = indent(*indent_level, self.indent_size, self.indent_style);
        write!(out, "{}", indent_str)?;

        // Write template parameters if any
        if !input.template_params.is_empty() {
            write!(out, "template<")?;

            for (i, param) in input.template_params.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ")?;
                }

                // Write keyword if it's not empty
                if !param.keyword.is_empty() {
                    write!(out, "{} ", param.keyword)?;
                } else if options.use_typename_default {
                    // If no keyword is specified but we're using default keywords
                    write!(out, "typename ")?;
                }

                // Write parameter name
                write!(out, "{}", param.name)?;

                // Write default value if present
                if let Some(default_value) = &param.default_value {
                    write!(out, " = {}", default_value)?;
                }
            }

            write!(out, ">{}{}", self.new_line.as_str(), indent_str)?;
        }

        if input.is_extern_c {
            write!(out, "extern \"C\" ")?;
        }

        // Write function modifiers. `friend`/`static`/`virtual`/`inline` are written
        // only for declarations (an out-of-line definition may not repeat them).
        if !options.render_definition {
            if input.is_friend {
                write!(out, "friend ")?;
            }

            if input.is_inline {
                write!(out, "inline ")?;
            }

            if input.is_static {
                write!(out, "static ")?;
            }

            if input.is_virtual {
                write!(out, "/* virtual */ ")?;
            }
        } else if options.inline_definition {
            // An out-of-line member-template definition emitted into a header must be
            // `inline` to avoid ODR violations across the many translation units that
            // include it.
            write!(out, "inline ")?;
        }

        // Write return type
        if let Some(return_type) = &input.return_type
            && !return_type.is_empty()
        {
            write!(out, "{} ", return_type)?;
        }

        // Write function name
        if options.render_definition
            && let Some(parent_name) = &input.parent_name
        {
            write!(out, "{}::", parent_name)?;
        }

        write!(out, "{}", input.name)?;

        // Write parameters
        write!(out, "(")?;
        for (i, param) in input.parameters.iter().enumerate() {
            if i > 0 {
                write!(out, ", ")?;
            }
            write!(out, "{} {}", param.param_type, param.name)?;
            if let Some(default_value) = &param.default_value {
                write!(out, " = {}", default_value)?;
            }
        }
        write!(out, ")")?;

        // Write const qualifier
        if input.is_const {
            write!(out, " const")?;
        }

        // Write noexcept qualifier
        if input.is_noexcept {
            write!(out, " noexcept")?;
        }

        // Write override keyword
        if input.is_override {
            write!(out, " override")?;
        }

        // Write final keyword
        if input.is_final {
            write!(out, " final")?;
        }

        // Write deleted functions
        if input.is_deleted {
            write!(out, " = delete")?;
        }

        // Write defaulted functions
        if input.is_default {
            write!(out, " = default")?;
        }

        // Write pure virtual
        if input.is_pure_virtual {
            write!(out, " = 0")?;
        }

        // Either write a semicolon for declarations or open a function body for definitions
        let base_skip: bool = !options.render_definition
            || input.is_pure_virtual
            || input.is_deleted
            || input.is_default
            || (input.is_friend && options.render_body_if_friend);
        let template_ok_to_skip: bool = input.template_params.is_empty() || !options.render_body_if_template;
        let friend_ok_to_skip: bool = !input.is_friend || !options.render_body_if_friend;
        if !options.force_render_body && base_skip && template_ok_to_skip && friend_ok_to_skip {
            write!(out, ";")?;
        } else {
            // Render function body for definitions
            if let Some(body_lines) = &input.body {
                // Use the actual function body if available
                if self.open_brace_on_new_line {
                    write!(out, "{0}{1}{{{0}", self.new_line.as_str(), indent_str)?
                } else {
                    write!(out, " {{{0}", self.new_line.as_str())?
                }

                // Process each line of the body with proper indentation
                // Increase indent level for function body
                let body_indent_str: String = indent(*indent_level + 1, self.indent_size, self.indent_style);

                for line in body_lines {
                    if !line.trim().is_empty() {
                        write!(out, "{}{}{}", body_indent_str, line, self.new_line.as_str())?;
                    } else {
                        write!(out, "{}", self.new_line.as_str())?;
                    }
                }

                write!(out, "{}}}", indent_str)?;
            } else {
                // Use placeholder for empty body
                // Increase indent level for function body
                let body_indent_str: String = indent(*indent_level + 1, self.indent_size, self.indent_style);

                if self.open_brace_on_new_line {
                    write!(
                        out,
                        "{0}{1}{{{0}{2}// Function body{0}{1}}}",
                        self.new_line.as_str(),
                        indent_str,
                        body_indent_str
                    )?
                } else {
                    write!(
                        out,
                        " {{{0}{1}// Function body{0}{2}}}",
                        self.new_line.as_str(),
                        body_indent_str,
                        indent_str
                    )?
                };
            }
        }

        // Write 'after' content if provided
        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }

        Ok(())
    }
}

impl StructRenderer for CppBackend {
    type StructType = CppStruct;
    type RenderOptions = CppStructRenderOptions;

    fn render_struct_to<S: AsRef<str>>(
        &self,
        input: &Self::StructType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl Write,
    ) -> Result<(), io::Error> {
        let binding = <CppBackend as StructRenderer>::default_options();
        let options: &CppStructRenderOptions = options.unwrap_or(&binding);

        // Write before string if provided
        if let Some(before) = before {
            write!(out, "{}", before.as_ref())?;
        }

        // Add documentation if available and render_docs is enabled
        if options.render_docs
            && let Some(docs) = &input.docs
        {
            self.write_docs(docs, indent_level, out)?;
        }

        let indent_str: String = indent(*indent_level, self.indent_size, self.indent_style);

        // Write template parameters if any
        if !input.template_params.is_empty() && options.render_template_params {
            write!(out, "{}template<", indent_str)?;

            for (i, param) in input.template_params.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ")?;
                }

                // Write keyword if it's not empty and we should render keywords
                if !param.keyword.is_empty() && options.render_template_param_keywords {
                    write!(out, "{} ", param.keyword)?;
                } else if options.use_typename_default && options.render_template_param_keywords {
                    // If no keyword is specified but we're using default keywords
                    write!(out, "typename ")?;
                }

                // Write parameter name
                write!(out, "{}", param.name)?;

                // Write default value if present
                if let Some(default_value) = &param.default_value {
                    write!(out, " = {}", default_value)?;
                }
            }

            write!(out, ">{}", self.new_line.as_str())?;
        }

        // Write struct declaration
        let alignas_prefix: String = match input.alignment {
            Some(n) => format!("alignas({}) ", n),
            None => String::new(),
        };
        match input.struct_kind {
            CppStructKind::Class => write!(out, "{}class {}{}", indent_str, alignas_prefix, input.name)?,
            CppStructKind::Struct => write!(out, "{}struct {}{}", indent_str, alignas_prefix, input.name)?,
            CppStructKind::Union => write!(out, "{}union {}{}", indent_str, alignas_prefix, input.name)?,
        }

        if input.is_final {
            write!(out, " final")?;
        }

        // Write inheritance list if any
        if !input.bases.is_empty() {
            write!(out, " : ")?;

            // Join bases with comma and space
            for (i, base) in input.bases.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ")?;
                }

                // Map visibility to C++ inheritance access specifier. `Default` mirrors C++'s own
                // default, which depends on the aggregate kind: `private` for a class, `public` for
                // a struct/union.
                let access_specifier = match base.visibility {
                    CppVisibility::Public => "public",
                    CppVisibility::Protected => "protected",
                    CppVisibility::Private => "private",
                    CppVisibility::Default => match input.struct_kind {
                        CppStructKind::Class => "private",
                        CppStructKind::Struct | CppStructKind::Union => "public",
                    },
                };

                write!(out, "{} {}", access_specifier, base.name)?;
            }
        }

        // Write struct body start
        if self.open_brace_on_new_line {
            write!(
                out,
                "{}{}{{{}",
                self.new_line.as_str(),
                indent(*indent_level, self.indent_size, self.indent_style),
                self.new_line.as_str(),
            )?
        } else {
            write!(out, " {{{}", self.new_line.as_str())?
        }

        // Increase indent level for struct body
        *indent_level += 1;

        // Process fields
        if !input.fields.is_empty() {
            // Track the current visibility to only print visibility sections when they change
            let mut current_visibility: CppVisibility = CppVisibility::Default;

            // If alignment is enabled, calculate max widths for each component
            let mut max_type_width: usize = 0;
            let mut max_name_width: usize = 0;

            if options.align_fields {
                // Calculate max widths from all fields
                for field in &input.fields {
                    let mut type_width: usize = 0;
                    if let Some(n) = field.alignment {
                        type_width += format!("alignas({}) ", n).len();
                    }
                    if field.is_inline {
                        type_width += "inline ".len();
                    }
                    if field.is_static {
                        type_width += "static ".len();
                    }
                    if field.is_const {
                        type_width += "const ".len();
                    }

                    type_width += field.field_type.len();

                    // Calculate name width including semicolon, array brackets, and bit field if present
                    let mut name_width: usize = field.name.len();

                    // Add array size width if present
                    if let Some(array_size) = &field.array_size {
                        // [ + array_size + ]
                        name_width += array_size.len() + 2;
                    }

                    // Add bit field width if present
                    if let Some(bit_size) = &field.bit_field_size {
                        // : + space + bit_size
                        name_width += bit_size.len() + 2;
                    }

                    // Add semicolon
                    name_width += 1;

                    max_type_width = max_type_width.max(type_width);
                    max_name_width = max_name_width.max(name_width);
                }
            }

            for (index, field) in input.fields.iter().enumerate() {
                // Compare visibilities, handling the case where Default might be effectively the same as Public/Private
                let effective_visibility = match field.visibility {
                    CppVisibility::Default => match input.struct_kind {
                        CppStructKind::Class => CppVisibility::Private,
                        CppStructKind::Union => CppVisibility::Private,
                        CppStructKind::Struct => CppVisibility::Public,
                    },
                    _ => field.visibility,
                };

                let effective_current_visibility = match current_visibility {
                    CppVisibility::Default => match input.struct_kind {
                        CppStructKind::Class => CppVisibility::Private,
                        CppStructKind::Union => CppVisibility::Private,
                        CppStructKind::Struct => CppVisibility::Public,
                    },
                    _ => current_visibility,
                };

                let should_print_visibility: bool = effective_visibility != effective_current_visibility
                    || (index == 0 && options.render_default_visibility);
                if should_print_visibility {
                    // Add a newline between sections (except before the first section)
                    if index > 0 {
                        write!(out, "{}", self.new_line.as_str())?;
                    }

                    // Update current visibility
                    current_visibility = field.visibility;

                    // Print visibility section header
                    let default_visibility: &str = match current_visibility {
                        CppVisibility::Public => "public",
                        CppVisibility::Protected => "protected",
                        CppVisibility::Private => "private",
                        CppVisibility::Default => match input.struct_kind {
                            CppStructKind::Class => "private",
                            CppStructKind::Union => "private",
                            CppStructKind::Struct => "public",
                        },
                    };
                    write!(
                        out,
                        "{}{}:{}",
                        indent(*indent_level - 1, self.indent_size, self.indent_style),
                        default_visibility,
                        self.new_line.as_str()
                    )?
                }

                // Add field documentation if available and render_docs is enabled
                if options.field_options.render_docs
                    && let Some(docs) = &field.docs
                {
                    self.write_docs(docs, indent_level, out)?;
                }

                // Build field declaration components
                let mut type_part = String::new();
                if let Some(n) = field.alignment {
                    type_part.push_str(&format!("alignas({}) ", n));
                }
                if field.is_inline {
                    type_part.push_str("inline ");
                }
                if field.is_static {
                    type_part.push_str("static ");
                }
                if field.is_const {
                    type_part.push_str("const ");
                }
                type_part.push_str(&field.field_type);

                // Write field declaration with proper alignment if enabled
                if options.align_fields {
                    // Write type with padding
                    write!(
                        out,
                        "{}{:<width$} ",
                        indent(*indent_level, self.indent_size, self.indent_style),
                        type_part,
                        width = max_type_width
                    )?;

                    // Write name with array size and/or bit field if any
                    let mut name_part = String::new();

                    // Add the field name
                    name_part.push_str(&field.name);

                    // Add array size if present
                    if let Some(array_size) = &field.array_size {
                        name_part.push_str(&format!("[{}]", array_size));
                    }

                    // Add bit field if present
                    if let Some(bit_size) = &field.bit_field_size {
                        name_part.push_str(&format!(": {}", bit_size));
                    }

                    // Add initialization value if present and enabled in options
                    if let Some(init_value) = &field.initialization_value
                        && options.field_options.render_initializers
                    {
                        name_part.push_str(&format!(" = {}", init_value));
                    }

                    // Add semicolon
                    name_part.push(';');

                    // Include the semicolon in the padding calculation
                    write!(out, "{:<width$}", name_part, width = max_name_width)?;

                    // Write inline comment if any
                    if let Some(comment) = &field.inline_comment {
                        write!(out, " // {}", comment)?;
                    }
                } else {
                    // No alignment - write normally
                    let mut field_decl = String::with_capacity(type_part.len() + field.name.len() + 10);
                    field_decl.push_str(&type_part);
                    field_decl.push(' ');
                    field_decl.push_str(&field.name);

                    // Add array size if present
                    if let Some(array_size) = &field.array_size {
                        field_decl.push_str(&format!("[{}]", array_size));
                    }

                    // Add bit field if present
                    if let Some(bit_size) = &field.bit_field_size {
                        field_decl.push_str(&format!(" : {}", bit_size));
                    }

                    // Add initialization value if present and enabled in options
                    if let Some(init_value) = &field.initialization_value
                        && options.field_options.render_initializers
                    {
                        field_decl.push_str(&format!(" = {}", init_value));
                    }

                    // Add semicolon
                    field_decl.push(';');

                    write!(
                        out,
                        "{}{}",
                        indent(*indent_level, self.indent_size, self.indent_style),
                        field_decl
                    )?;

                    // Add inline comment if available
                    if let Some(comment) = &field.inline_comment {
                        write!(out, " // {}", comment)?;
                    }
                }

                write!(out, "{}", self.new_line.as_str())?;
            }
        }

        // Process methods
        if !input.methods.is_empty() {
            // Add a separator between fields and methods if there are fields
            if !input.fields.is_empty() {
                write!(out, "{}", self.new_line.as_str())?;
            }

            // Track the current visibility to know when to print visibility labels
            let mut current_visibility: CppVisibility = CppVisibility::Default;

            for (index, method) in input.methods.iter().enumerate() {
                let is_first_method: bool = index == 0;
                let is_last_method: bool = index == input.methods.len() - 1;

                // Compare visibilities, handling the case where Default might be effectively the same as Public/Private
                let effective_visibility: CppVisibility = match method.visibility {
                    CppVisibility::Default => match input.struct_kind {
                        CppStructKind::Class => CppVisibility::Private,
                        CppStructKind::Union => CppVisibility::Private,
                        CppStructKind::Struct => CppVisibility::Public,
                    },
                    _ => method.visibility,
                };

                let effective_current_visibility: CppVisibility = match current_visibility {
                    CppVisibility::Default => match input.struct_kind {
                        CppStructKind::Class => CppVisibility::Private,
                        CppStructKind::Union => CppVisibility::Private,
                        CppStructKind::Struct => CppVisibility::Public,
                    },
                    _ => current_visibility,
                };

                let should_print_visibility: bool = effective_visibility != effective_current_visibility
                    || (is_first_method && options.render_default_visibility);
                if should_print_visibility {
                    // Decrease indent for visibility label
                    *indent_level -= 1;

                    // Write the new visibility label
                    let visibility_str: &str = match method.visibility {
                        CppVisibility::Public => "public",
                        CppVisibility::Protected => "protected",
                        CppVisibility::Private => "private",
                        CppVisibility::Default => match input.struct_kind {
                            CppStructKind::Class => "private",
                            CppStructKind::Union => "private",
                            CppStructKind::Struct => "public",
                        },
                    };

                    write!(
                        out,
                        "{}{}:{}",
                        indent(*indent_level, self.indent_size, self.indent_style),
                        visibility_str,
                        self.new_line.as_str()
                    )?;

                    // Update current visibility
                    current_visibility = method.visibility;

                    // Increase indent again for methods
                    *indent_level += 1;
                }

                // Render the method
                self.render_function_to::<&str>(method, None, None, Some(&options.method_options), indent_level, out)?;

                if !is_last_method {
                    write!(out, "{}", self.new_line.as_str())?;
                }
            }
        }

        // Decrease indent after struct body
        *indent_level -= 1;

        // Write struct body end
        write!(
            out,
            "{}{}}};{}",
            self.new_line.as_str(),
            indent(*indent_level, self.indent_size, self.indent_style),
            self.new_line.as_str()
        )?;

        // Write after string if provided
        if let Some(after) = after {
            write!(out, "{}", after.as_ref())?;
        }

        Ok(())
    }
}

impl CppBackend {
    fn write_docs(&self, docs: &Vec<String>, indent_level: &mut i32, out: &mut impl Write) -> Result<(), io::Error> {
        for line in docs {
            match self.docs_style {
                DocsStyle::DoubleSlash => write!(
                    out,
                    "{}// {}{}",
                    indent(*indent_level, self.indent_size, self.indent_style),
                    line,
                    self.new_line.as_str()
                )?,
                DocsStyle::TripleSlash => write!(
                    out,
                    "{}/// {}{}",
                    indent(*indent_level, self.indent_size, self.indent_style),
                    line,
                    self.new_line.as_str()
                )?,
                DocsStyle::SlashAsterisk => write!(
                    out,
                    "{}/*{}*/{}",
                    indent(*indent_level, self.indent_size, self.indent_style),
                    line,
                    self.new_line.as_str()
                )?, // TODO: This is wrong and will print for every line
                DocsStyle::SlashDoubleAsterisk => write!(
                    out,
                    "{}/**{}*/{}",
                    indent(*indent_level, self.indent_size, self.indent_style),
                    line,
                    self.new_line.as_str()
                )?, // TODO: This is wrong and will print for every line
            };
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::cpp_backend::struct_types::CppStructKind;
    use crate::backends::cpp_backend::{CppField, CppStruct};

    fn test_backend() -> CppBackend {
        CppBackend {
            new_line: NewLineStyle::LF,
            open_brace_on_new_line: false,
            docs_style: DocsStyle::DoubleSlash,
            indent_style: IndentStyle::Spaces,
            indent_size: 4,
        }
    }

    fn plain_field(name: &str, field_type: &str, alignment: Option<u32>) -> CppField {
        CppField {
            name: name.to_string(),
            field_type: field_type.to_string(),
            visibility: CppVisibility::Public,
            array_size: None,
            bit_field_size: None,
            alignment,
            is_static: false,
            is_const: false,
            is_inline: false,
            initialization_value: None,
            inline_comment: None,
            docs: None,
        }
    }

    fn struct_with(alignment: Option<u32>, fields: Vec<CppField>) -> CppStruct {
        CppStruct {
            struct_kind: CppStructKind::Struct,
            is_final: false,
            alignment,
            name: "Foo".to_string(),
            template_params: Vec::new(),
            bases: Vec::new(),
            fields,
            methods: Vec::new(),
            docs: None,
        }
    }

    #[test]
    fn renders_alignas_on_field() {
        let backend: CppBackend = test_backend();
        let input: CppStruct = struct_with(
            None,
            vec![
                plain_field("normal", "int", None),
                plain_field("cctor_thread", "size_t", Some(8)),
            ],
        );

        let mut indent_level: i32 = 0;
        let output: String = backend
            .render_struct::<&str>(&input, None, None, None, &mut indent_level)
            .expect("render struct");

        assert!(
            output.contains("alignas(8) size_t cctor_thread;"),
            "output was: {output}"
        );
    }

    #[test]
    fn renders_alignas_on_struct() {
        let backend: CppBackend = test_backend();
        let input: CppStruct = struct_with(Some(16), vec![plain_field("normal", "int", None)]);

        let mut indent_level: i32 = 0;
        let output: String = backend
            .render_struct::<&str>(&input, None, None, None, &mut indent_level)
            .expect("render struct");

        assert!(output.contains("struct alignas(16) Foo"), "output was: {output}");
    }

    fn free_function(name: &str, is_extern_c: bool) -> CppFunction {
        CppFunction {
            name: name.to_string(),
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
            is_extern_c,
            is_override: false,
            is_final: false,
            is_friend: false,
            is_deleted: false,
            is_default: false,
            body: None,
            docs: None,
        }
    }

    #[test]
    fn renders_extern_c_specifier() {
        let backend: CppBackend = test_backend();
        let input = free_function("polyplug_init", true);

        let mut indent_level: i32 = 0;
        let output: String = backend
            .render_function::<&str>(&input, None, None, None, &mut indent_level)
            .expect("render function");

        assert!(output.contains("extern \"C\" void polyplug_init("), "output was: {output}");
    }

    #[test]
    fn normal_function_omits_extern_c() {
        let backend: CppBackend = test_backend();
        let input = free_function("plain", false);

        let mut indent_level: i32 = 0;
        let output: String = backend
            .render_function::<&str>(&input, None, None, None, &mut indent_level)
            .expect("render function");

        assert!(!output.contains("extern"), "output was: {output}");
    }
}
