use crate::{
    backends::BackendItem,
    conversion::ConversionResult,
    convert::ConversionConfig,
    ir::{LanguageField, LanguageStruct, LanguageStructKind, Visibility},
};

/// A field of a ctypes `Structure`: a name paired with a free-form ctype string.
///
/// The ctype is intentionally a raw string (e.g. `ctypes.c_int32`) — Python's
/// ctypes vocabulary is open-ended and the backend renders form, not semantics.
#[derive(Debug, Clone, PartialEq)]
pub struct PythonStructField {
    /// The name of the field.
    pub name: String,
    /// The ctype expression, rendered verbatim (e.g. `ctypes.c_int32`).
    pub ctype: String,
}

/// Represents a ctypes structure: `class Name(ctypes.Structure):` with a
/// `_fields_` class attribute.
#[derive(Debug, Clone, PartialEq)]
pub struct PythonStruct {
    /// The name of the structure.
    pub name: String,
    /// The fields, rendered into the `_fields_` list as `("name", ctype)` tuples.
    pub fields: Vec<PythonStructField>,
    /// Optional docstring, rendered as the first triple-quoted body line.
    pub docstring: Option<String>,
}

impl BackendItem for PythonStruct {
    type IrType = LanguageStruct;
    type ConversionOptions = PythonStructConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let fields = self
            .fields
            .into_iter()
            .map(|field| LanguageField {
                name: field.name,
                field_type: field.ctype,
                visibility: Visibility::Public,
                is_static: false,
                is_const: false,
                docs: None,
                annotations: Vec::new(),
                raw_attributes: Vec::new(),
            })
            .collect();

        ConversionResult::new(LanguageStruct {
            visibility: Visibility::Public,
            struct_kind: LanguageStructKind::Struct,
            is_abstract: false,
            is_final: false,
            name: self.name,
            generic_args: Vec::new(),
            bases: Vec::new(),
            fields,
            methods: Vec::new(),
            docs: self.docstring.map(|docstring| vec![docstring]),
            annotations: Vec::new(),
            raw_attributes: Vec::new(),
        })
    }

    fn from_ir(input: Self::IrType, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let fields = input
            .fields
            .into_iter()
            .map(|field| PythonStructField {
                name: field.name,
                ctype: field.field_type,
            })
            .collect();

        ConversionResult::new(PythonStruct {
            name: input.name,
            fields,
            docstring: input.docs.map(|docs| docs.join("\n")),
        })
    }
}

/// Conversion options for Python ctypes structures.
#[derive(Debug, Clone, Default)]
pub struct PythonStructConversionOptions {
    /// Cross-language conversion configuration.
    pub config: ConversionConfig,
}

/// Render options for Python ctypes structures.
#[derive(Debug, Clone)]
pub struct PythonStructRenderOptions {
    /// Whether to render the docstring.
    pub render_docstring: bool,
}

impl Default for PythonStructRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl PythonStructRenderOptions {
    pub const DEFAULT: Self = Self { render_docstring: true };
}
