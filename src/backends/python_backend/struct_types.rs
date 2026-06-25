use crate::{
    backends::BackendItem,
    conversion::{dropped_annotations_warning, ConversionLog, ConversionResult, ConversionWarning},
    convert::{rename_identifier, ConversionConfig, IdentifierKind},
    ir::{LanguageField, LanguageStruct, LanguageStructKind, Visibility},
    type_map::{PrimitiveType, TargetLanguage},
};

/// Map a neutral [`PrimitiveType`] to its ctypes spelling.
///
/// `ctypes` is not a [`TargetLanguage`] — it is a Python-local FFI vocabulary, so this mapping
/// lives here rather than in the shared [`TypeMap`](crate::type_map::TypeMap) output table. Returns
/// `None` for primitives ctypes has no native spelling for ([`PrimitiveType::I128`]/[`U128`](PrimitiveType::U128))
/// and for [`PrimitiveType::Void`], which is not a field type.
fn ctype_for(primitive: PrimitiveType) -> Option<&'static str> {
    use PrimitiveType::*;
    Some(match primitive {
        Bool => "ctypes.c_bool",
        I8 => "ctypes.c_int8",
        U8 => "ctypes.c_uint8",
        I16 => "ctypes.c_int16",
        U16 => "ctypes.c_uint16",
        I32 => "ctypes.c_int32",
        U32 => "ctypes.c_uint32",
        I64 => "ctypes.c_int64",
        U64 => "ctypes.c_uint64",
        ISize => "ctypes.c_ssize_t",
        USize => "ctypes.c_size_t",
        F32 => "ctypes.c_float",
        F64 => "ctypes.c_double",
        Char => "ctypes.c_char",
        Str => "ctypes.c_char_p",
        I128 | U128 | Void => return None,
    })
}

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

    fn from_ir(input: Self::IrType, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();
        let config = options.map(|options| options.config.clone()).unwrap_or_default();

        let name = rename_identifier(&config, &input.name, TargetLanguage::Python, IdentifierKind::Type);
        log.add_warnings(name.log.warnings);

        if !input.annotations.is_empty() || !input.raw_attributes.is_empty() {
            log.add_warning(dropped_annotations_warning(
                input.annotations.len() + input.raw_attributes.len(),
                "struct",
                &input.name,
                "Python",
            ));
        }

        let mut fields = Vec::with_capacity(input.fields.len());
        for field in input.fields {
            let field_name = rename_identifier(&config, &field.name, TargetLanguage::Python, IdentifierKind::Field);
            log.add_warnings(field_name.log.warnings);

            let ctype = match config.type_map.resolve(&field.field_type) {
                Some(primitive) => match ctype_for(primitive) {
                    Some(ctype) => ctype.to_string(),
                    None => {
                        log.add_warning(ConversionWarning::UnsupportedFeature {
                            feature: format!("field `{}` of type `{}`", field.name, field.field_type),
                            resolution: "ctypes has no native type for it; emitted the spelling verbatim".to_string(),
                        });
                        field.field_type
                    }
                },
                None => field.field_type,
            };

            fields.push(PythonStructField {
                name: field_name.value,
                ctype,
            });
        }

        ConversionResult::with_log(
            PythonStruct {
                name: name.value,
                fields,
                docstring: input.docs.map(|docs| docs.join("\n")),
            },
            log,
        )
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
