use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult},
    convert::ConversionConfig,
    ir::{LanguageBase, LanguageField, LanguageStruct, LanguageStructKind, Visibility},
};

use super::{PythonFunction, PythonFunctionConversionOptions};

/// A class-level field assignment (`name = value`).
///
/// Python class attributes are plain assignments with a free-form right-hand
/// side, so both name and value are modelled as raw strings.
#[derive(Debug, Clone, PartialEq)]
pub struct PythonClassField {
    /// The name of the attribute.
    pub name: String,
    /// The assigned value, rendered verbatim (e.g. `0`, `"default"`, `None`).
    pub value: String,
}

/// Represents a plain Python class (`class Name:` / `class Name(Base):`).
#[derive(Debug, Clone, PartialEq)]
pub struct PythonClass {
    /// The name of the class.
    pub name: String,
    /// Base classes, rendered verbatim in the parenthesised base list.
    pub bases: Vec<String>,
    /// Class-level field assignments.
    pub fields: Vec<PythonClassField>,
    /// The methods (`def`) of the class.
    pub methods: Vec<PythonFunction>,
    /// Optional class docstring, rendered as the first triple-quoted body line.
    pub docstring: Option<String>,
}

impl BackendItem for PythonClass {
    type IrType = LanguageStruct;
    type ConversionOptions = PythonClassConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut log = ConversionLog::new();

        let fields = self
            .fields
            .into_iter()
            .map(|field| LanguageField {
                name: field.name,
                field_type: String::new(),
                visibility: Visibility::Public,
                is_static: true,
                is_const: false,
                docs: None,
                annotations: Vec::new(),
                raw_attributes: Vec::new(),
            })
            .collect();

        let mut methods = Vec::with_capacity(self.methods.len());
        for method in self.methods {
            let result = method.to_ir(None);
            log.add_warnings(result.log.warnings);
            methods.push(result.value);
        }

        let bases = self
            .bases
            .into_iter()
            .map(|name| LanguageBase {
                name,
                visibility: Visibility::Public,
            })
            .collect();

        ConversionResult::with_log(
            LanguageStruct {
                visibility: Visibility::Public,
                struct_kind: LanguageStructKind::Class,
                is_abstract: false,
                is_final: false,
                name: self.name,
                generic_args: Vec::new(),
                bases,
                fields,
                methods,
                docs: self.docstring.map(|docstring| vec![docstring]),
                annotations: Vec::new(),
                raw_attributes: Vec::new(),
            },
            log,
        )
    }

    fn from_ir(input: Self::IrType, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();
        let config = options.map(|options| options.config.clone()).unwrap_or_default();

        let fields = input
            .fields
            .into_iter()
            .map(|field| PythonClassField {
                name: field.name,
                value: "None".to_string(),
            })
            .collect();

        let function_options = PythonFunctionConversionOptions { config };
        let mut methods = Vec::with_capacity(input.methods.len());
        for method in input.methods {
            let result = PythonFunction::from_ir(method, Some(&function_options));
            log.add_warnings(result.log.warnings);
            methods.push(result.value);
        }

        let bases = input.bases.into_iter().map(|base| base.name).collect();

        ConversionResult::with_log(
            PythonClass {
                name: input.name,
                bases,
                fields,
                methods,
                docstring: input.docs.map(|docs| docs.join("\n")),
            },
            log,
        )
    }
}

/// Conversion options for Python classes.
#[derive(Debug, Clone, Default)]
pub struct PythonClassConversionOptions {
    /// Cross-language conversion configuration.
    pub config: ConversionConfig,
}

/// Render options for Python classes.
#[derive(Debug, Clone)]
pub struct PythonClassRenderOptions {
    /// Whether to render the docstring.
    pub render_docstring: bool,
}

impl Default for PythonClassRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl PythonClassRenderOptions {
    pub const DEFAULT: Self = Self { render_docstring: true };
}
