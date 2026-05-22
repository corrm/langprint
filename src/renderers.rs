use std::{
    io::{self},
    sync::LazyLock,
};

use crate::backends::{BackendItem, BackendMetadata};

/// Trait for rendering defines.
#[allow(clippy::declare_interior_mutable_const)]
pub trait DefinitionRenderer: BackendMetadata {
    /// The type of define to render.
    type DefineType: BackendItem;
    /// The render options type for this renderer.
    type RenderOptions: Default;

    /// The default render options for this renderer.
    const DEFAULT_RENDER_OPTIONS: LazyLock<Self::RenderOptions> = LazyLock::new(Self::RenderOptions::default);

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
        let mut output: Vec<u8> = Vec::new();
        self.render_definition_to(input, before, after, options, indent_level, &mut output)?;
        Ok(String::from_utf8(output).expect("Rendered output is not valid UTF-8"))
    }
}

// Trait for rendering namespaces.
#[allow(clippy::declare_interior_mutable_const)]
pub trait NamespaceRenderer: BackendMetadata {
    /// The namespace type for this renderer.
    type NamespaceType: BackendItem;
    /// The render options type for this renderer.
    type RenderOptions: Default;

    /// The default render options for this renderer.
    const DEFAULT_RENDER_OPTIONS: LazyLock<Self::RenderOptions> = LazyLock::new(Self::RenderOptions::default);

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
        let mut output: Vec<u8> = Vec::new();
        self.render_namespace_to(input, before, after, options, indent_level, &mut output)?;
        Ok(String::from_utf8(output).expect("Rendered output is not valid UTF-8"))
    }
}

#[allow(clippy::declare_interior_mutable_const)]
/// Trait for rendering constants.
pub trait ConstantRenderer: BackendMetadata {
    /// The constant type for this renderer.
    type ConstantType: BackendItem;
    /// The render options type for this renderer.
    type RenderOptions: Default;

    /// The default render options for this renderer.
    const DEFAULT_RENDER_OPTIONS: LazyLock<Self::RenderOptions> = LazyLock::new(Self::RenderOptions::default);

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
        let mut output: Vec<u8> = Vec::new();
        self.render_constant_to(input, before, after, options, indent_level, &mut output)?;
        Ok(String::from_utf8(output).expect("Rendered output is not valid UTF-8"))
    }
}

#[allow(clippy::declare_interior_mutable_const)]
/// Trait for rendering functions.
pub trait FunctionRenderer: BackendMetadata {
    /// The function type for this renderer.
    type FunctionType: BackendItem;
    /// The render options type for this renderer.
    type RenderOptions: Default;

    /// The default render options for this renderer.
    const DEFAULT_RENDER_OPTIONS: LazyLock<Self::RenderOptions> = LazyLock::new(Self::RenderOptions::default);

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
        let mut output: Vec<u8> = Vec::new();
        self.render_function_to(input, before, after, options, indent_level, &mut output)?;
        Ok(String::from_utf8(output).expect("Rendered output is not valid UTF-8"))
    }
}

#[allow(clippy::declare_interior_mutable_const)]
/// Trait for rendering enums.
pub trait EnumRenderer: BackendMetadata {
    /// The enum type for this renderer.
    type EnumType: BackendItem;
    /// The variant render options type for this renderer.
    type EnumVariantRenderOptions: Default;
    /// The render options type for this renderer.
    type RenderOptions: Default;

    /// The default render options for this renderer.
    const DEFAULT_RENDER_OPTIONS: LazyLock<Self::RenderOptions> = LazyLock::new(Self::RenderOptions::default);

    /// The default variant render options for this renderer.
    const DEFAULT_ENUM_VARIANT_RENDER_OPTIONS: LazyLock<Self::EnumVariantRenderOptions> =
        LazyLock::new(Self::EnumVariantRenderOptions::default);

    /// Render a language-specific enum to a writer.
    ///
    /// # Arguments
    ///
    /// * `input` - The enum to render.
    /// * `before` - The string to prepend to the enum.
    /// * `after` - The string to append to the enum.
    /// * `options` - The render options to use, if None, default options will be used.
    /// * `variant_options` - The variant render options to use, if None, default options will be used.
    /// * `out` - The writer to render to.
    fn render_enum_to<S: AsRef<str>>(
        &self,
        input: &Self::EnumType,
        before: Option<S>,
        after: Option<S>,
        options: Option<&Self::RenderOptions>,
        variant_options: Option<&Self::EnumVariantRenderOptions>,
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
    /// * `options` - The render options to use, if None, default options will be used.
    /// * `variant_options` - The variant render options to use, if None, default options will be used.
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
        variant_options: Option<&Self::EnumVariantRenderOptions>,
        indent_level: &mut i32,
    ) -> Result<String, io::Error> {
        let mut output: Vec<u8> = Vec::new();
        self.render_enum_to(
            input,
            before,
            after,
            options,
            variant_options,
            indent_level,
            &mut output,
        )?;
        Ok(String::from_utf8(output).expect("Rendered output is not valid UTF-8"))
    }
}

#[allow(clippy::declare_interior_mutable_const)]
/// Trait for rendering structs.
pub trait StructRenderer: BackendMetadata {
    /// The struct type for this renderer.
    type StructType: BackendItem;
    /// The render options type for this renderer.
    type RenderOptions: Default;

    /// The default render options for this renderer.
    const DEFAULT_RENDER_OPTIONS: LazyLock<Self::RenderOptions> = LazyLock::new(Self::RenderOptions::default);

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
        let mut output: Vec<u8> = Vec::new();
        self.render_struct_to(input, before, after, options, indent_level, &mut output)?;
        Ok(String::from_utf8(output).expect("Rendered output is not valid UTF-8"))
    }
}

#[allow(clippy::declare_interior_mutable_const)]
/// Trait for rendering interfaces.
pub trait InterfaceRenderer: BackendMetadata {
    /// The interface type for this renderer.
    type InterfaceType: BackendItem;
    /// The render options type for this renderer.
    type RenderOptions: Default;

    /// The default render options for this renderer.
    const DEFAULT_RENDER_OPTIONS: LazyLock<Self::RenderOptions> = LazyLock::new(Self::RenderOptions::default);

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
        let mut output: Vec<u8> = Vec::new();
        self.render_interface_to(input, before, after, options, indent_level, &mut output)?;
        Ok(String::from_utf8(output).expect("Rendered output is not valid UTF-8"))
    }
}
