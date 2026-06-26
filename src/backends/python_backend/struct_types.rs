use crate::{
    backends::BackendItem,
    conversion::{
        ConversionLog, ConversionResult, dropped_annotations_warning, dropped_feature_warning,
    },
    convert::{ConversionConfig, IdentifierKind, map_type, rename_identifier},
    ir::{LanguageField, LanguageStruct, LanguageStructKind, Visibility},
    type_map::TargetLanguage,
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

    fn to_ir(self, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
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

        let mut ir = LanguageStruct {
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
        };
        if let Some(hooks) = options.and_then(|options| options.config.hooks.as_ref()) {
            hooks.after_to_ir_struct(&mut ir);
        }

        ConversionResult::new(ir)
    }

    fn from_ir(
        mut input: Self::IrType,
        options: Option<&Self::ConversionOptions>,
    ) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();
        let config = options.map(|o| o.config.clone()).unwrap_or_default();
        if let Some(hooks) = &config.hooks {
            hooks.before_from_ir_struct(&mut input);
        }

        if !input.generic_args.is_empty() {
            log.add_warning(dropped_feature_warning(
                "generic arguments",
                &input.name,
                "Python",
            ));
        }

        let name = rename_identifier(
            &config,
            &input.name,
            TargetLanguage::Python,
            IdentifierKind::Type,
        );
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
            let field_name = rename_identifier(
                &config,
                &field.name,
                TargetLanguage::Python,
                IdentifierKind::Field,
            );
            log.add_warnings(field_name.log.warnings);

            let mapped = map_type(&config, &field.field_type, TargetLanguage::Python);
            log.add_warnings(mapped.log.warnings);

            fields.push(PythonStructField {
                name: field_name.value,
                ctype: mapped.value,
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
    pub const DEFAULT: Self = Self {
        render_docstring: true,
    };
}
