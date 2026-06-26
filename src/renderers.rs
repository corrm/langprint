use std::io;

use crate::backends::{BackendItem, BackendMetadata};

fn render_to_string<F>(f: F) -> Result<String, io::Error>
where
    F: FnOnce(&mut Vec<u8>) -> Result<(), io::Error>,
{
    let mut output = Vec::new();
    f(&mut output)?;
    Ok(String::from_utf8(output).expect("Rendered output is not valid UTF-8"))
}

/// Opt-in post-processing for rendered output.
///
/// Renderers return a `String` the caller owns; this applies an optional hook over that string,
/// for cases the renderer itself does not model — e.g. prepending a `#pragma once` file preamble.
/// With `None` the string is returned unchanged.
///
/// ```
/// use langprint::renderers::post_process;
///
/// let wrap = |s: String| format!("#pragma once\n{s}");
/// assert_eq!(post_process("body".to_string(), Some(&wrap)), "#pragma once\nbody");
/// assert_eq!(post_process("body".to_string(), None), "body");
/// ```
pub fn post_process(rendered: String, hook: Option<&dyn Fn(String) -> String>) -> String {
    match hook {
        Some(hook) => hook(rendered),
        None => rendered,
    }
}

/// Trait for rendering defines.
pub trait DefinitionRenderer: BackendMetadata {
    /// The type of define to render.
    type DefineType: BackendItem;
    /// The render options type for this renderer.
    type RenderOptions: Default;

    fn default_options() -> Self::RenderOptions { Self::RenderOptions::default() }

    /// Render a language-specific define to a writer.
    ///
    /// # Arguments
    ///
    /// * `input` - The define to render.
    /// * `before` - The string to prepend to the define.
    /// * `after` - The string to append to the define.
    /// * `options` - The render options to use, if None, default options will be used.
    ///
    /// # Returns
    ///
    /// A string containing the rendered define.
    fn render_definition_to<S: AsRef<str>>(
        &self,
        input: &Self::DefineType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl io::Write,
    ) -> Result<(), io::Error>;

    /// Render a language-specific define to a string.
    ///
    /// # Arguments
    ///
    /// * `input` - The define to render.
    /// * `before` - The string to prepend to the define.
    /// * `after` - The string to append to the define.
    /// * `options` - The render options to use, if None, default options will be used.
    ///
    /// # Returns
    ///
    /// A string containing the rendered define.
    fn render_definition<S: AsRef<str>>(
        &self,
        input: &Self::DefineType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
    ) -> Result<String, io::Error> {
        render_to_string(|out| self.render_definition_to(input, before, after, options, indent_level, out))
    }
}

// Trait for rendering namespaces.
pub trait NamespaceRenderer: BackendMetadata {
    /// The namespace type for this renderer.
    type NamespaceType: BackendItem;
    /// The render options type for this renderer.
    type RenderOptions: Default;

    fn default_options() -> Self::RenderOptions { Self::RenderOptions::default() }

    /// Render a language-specific namespace to a writer.
    ///
    /// # Arguments
    ///
    /// * `input` - The namespace to render.
    /// * `before` - The string to prepend to the namespace.
    /// * `after` - The string to append to the namespace.
    /// * `options` - The render options to use, if None, default options will be used.
    /// * `out` - The writer to render to.
    fn render_namespace_to<S: AsRef<str>>(
        &self,
        input: &Self::NamespaceType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl io::Write,
    ) -> Result<(), io::Error>;

    /// Render a language-specific namespace to a string.
    ///
    /// # Arguments
    ///
    /// * `input` - The namespace to render.
    /// * `before` - The string to prepend to the namespace.
    /// * `after` - The string to append to the namespace.
    /// * `options` - The render options to use, if None, default options will be used.
    ///
    /// # Returns
    ///
    /// A string containing the rendered namespace.
    fn render_namespace<S: AsRef<str>>(
        &self,
        input: &Self::NamespaceType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
    ) -> Result<String, io::Error> {
        render_to_string(|out| self.render_namespace_to(input, before, after, options, indent_level, out))
    }
}

/// Trait for rendering constants.
pub trait ConstantRenderer: BackendMetadata {
    /// The constant type for this renderer.
    type ConstantType: BackendItem;
    /// The render options type for this renderer.
    type RenderOptions: Default;

    fn default_options() -> Self::RenderOptions { Self::RenderOptions::default() }

    /// Render a language-specific constant to a writer.
    ///
    /// # Arguments
    ///
    /// * `input` - The constant to render.
    /// * `before` - The string to prepend to the constant.
    /// * `after` - The string to append to the constant.
    /// * `options` - The render options to use, if None, default options will be used.
    /// * `out` - The writer to render to.
    fn render_constant_to<S: AsRef<str>>(
        &self,
        input: &Self::ConstantType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl io::Write,
    ) -> Result<(), io::Error>;

    /// Render a language-specific constant to a string.
    ///
    /// # Arguments
    ///
    /// * `before` - The string to prepend to the constant.
    /// * `after` - The string to append to the constant.
    /// * `options` - The render options to use, if None, default options will be used.
    ///
    /// # Returns
    ///
    /// A string containing the rendered constant.
    fn render_constant<S: AsRef<str>>(
        &self,
        input: &Self::ConstantType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
    ) -> Result<String, io::Error> {
        render_to_string(|out| self.render_constant_to(input, before, after, options, indent_level, out))
    }
}

/// Trait for rendering functions.
///
/// Implementors honor the body-slot contract: the function type carries
/// `body: Option<Vec<String>>`. `None` renders a bare declaration terminated for the
/// language; `Some(lines)` renders the signature followed by a block whose lines are the
/// consumer's raw strings, emitted verbatim one indent deeper. langprint owns only the
/// indentation and block punctuation and never models statements or expressions.
pub trait FunctionRenderer: BackendMetadata {
    /// The function type for this renderer.
    type FunctionType: BackendItem;
    /// The render options type for this renderer.
    type RenderOptions: Default;

    fn default_options() -> Self::RenderOptions { Self::RenderOptions::default() }

    /// Render a language-specific function to a writer.
    ///
    /// # Arguments
    ///
    /// * `input` - The function to render.
    /// * `before` - The string to prepend to the function.
    /// * `after` - The string to append to the function.
    /// * `options` - The render options to use, if None, default options will be used.
    /// * `out` - The writer to render to.
    fn render_function_to<S: AsRef<str>>(
        &self,
        input: &Self::FunctionType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl io::Write,
    ) -> Result<(), io::Error>;

    /// Render a language-specific function to a string.
    ///
    /// # Arguments
    ///
    /// * `before` - The string to prepend to the function.
    /// * `after` - The string to append to the function.
    /// * `options` - The render options to use, if None, default options will be used.
    ///
    /// # Returns
    ///
    /// A string containing the rendered function.
    fn render_function<S: AsRef<str>>(
        &self,
        input: &Self::FunctionType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
    ) -> Result<String, io::Error> {
        render_to_string(|out| self.render_function_to(input, before, after, options, indent_level, out))
    }
}

/// Trait for rendering enums.
pub trait EnumRenderer: BackendMetadata {
    /// The enum type for this renderer.
    type EnumType: BackendItem;
    /// The render options type for this renderer. Variant render options are nested within it.
    type RenderOptions: Default;

    fn default_options() -> Self::RenderOptions { Self::RenderOptions::default() }

    /// Render a language-specific enum to a writer.
    ///
    /// # Arguments
    ///
    /// * `input` - The enum to render.
    /// * `before` - The string to prepend to the enum.
    /// * `after` - The string to append to the enum.
    /// * `options` - The render options to use, if None, default options will be used. Variant
    ///   render options are nested within it.
    /// * `out` - The writer to render to.
    fn render_enum_to<S: AsRef<str>>(
        &self,
        input: &Self::EnumType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl io::Write,
    ) -> Result<(), io::Error>;

    /// Render a language-specific enum to a string.
    ///
    /// # Arguments
    ///
    /// * `input` - The enum to render.
    /// * `before` - The string to prepend to the enum.
    /// * `after` - The string to append to the enum.
    /// * `options` - The render options to use, if None, default options will be used. Variant
    ///   render options are nested within it.
    ///
    /// # Returns
    ///
    /// A string containing the rendered enum.
    fn render_enum<S: AsRef<str>>(
        &self,
        input: &Self::EnumType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
    ) -> Result<String, io::Error> {
        render_to_string(|out| self.render_enum_to(input, before, after, options, indent_level, out))
    }
}

/// Trait for rendering structs.
pub trait StructRenderer: BackendMetadata {
    /// The struct type for this renderer.
    type StructType: BackendItem;
    /// The render options type for this renderer.
    type RenderOptions: Default;

    fn default_options() -> Self::RenderOptions { Self::RenderOptions::default() }

    /// Render a language-specific struct to a writer.
    ///
    /// # Arguments
    ///
    /// * `input` - The struct to render.
    /// * `before` - The string to prepend to the struct.
    /// * `after` - The string to append to the struct.
    /// * `options` - The render options to use, if None, default options will be used.
    /// * `out` - The writer to render to.
    fn render_struct_to<S: AsRef<str>>(
        &self,
        input: &Self::StructType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl io::Write,
    ) -> Result<(), io::Error>;

    /// Render a language-specific struct to a string.
    ///
    /// # Arguments
    ///
    /// * `input` - The struct to render.
    /// * `before` - The string to prepend to the struct.
    /// * `after` - The string to append to the struct.
    /// * `options` - The render options to use, if None, default options will be used.
    ///
    /// # Returns
    ///
    /// A string containing the rendered struct.
    fn render_struct<S: AsRef<str>>(
        &self,
        input: &Self::StructType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
    ) -> Result<String, io::Error> {
        render_to_string(|out| self.render_struct_to(input, before, after, options, indent_level, out))
    }
}

/// Trait for rendering interfaces.
pub trait InterfaceRenderer: BackendMetadata {
    /// The interface type for this renderer.
    type InterfaceType: BackendItem;
    /// The render options type for this renderer.
    type RenderOptions: Default;

    fn default_options() -> Self::RenderOptions { Self::RenderOptions::default() }

    /// Render a language-specific interface to a writer.
    ///
    /// # Arguments
    ///
    /// * `input` - The interface to render.
    /// * `before` - The string to prepend to the interface.
    /// * `after` - The string to append to the interface.
    /// * `options` - The render options to use, if None, default options will be used.
    /// * `out` - The writer to render to.
    fn render_interface_to<S: AsRef<str>>(
        &self,
        input: &Self::InterfaceType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
        out: &mut impl io::Write,
    ) -> Result<(), io::Error>;

    /// Render a language-specific interface to a string.
    ///
    /// # Arguments
    ///
    /// * `input` - The interface to render.
    /// * `before` - The string to prepend to the interface.
    /// * `after` - The string to append to the interface.
    /// * `options` - The render options to use, if None, default options will be used.
    ///
    /// # Returns
    ///
    /// A string containing the rendered interface.
    fn render_interface<S: AsRef<str>>(
        &self,
        input: &Self::InterfaceType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        indent_level: &mut i32,
    ) -> Result<String, io::Error> {
        render_to_string(|out| self.render_interface_to(input, before, after, options, indent_level, out))
    }
}
