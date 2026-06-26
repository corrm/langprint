use crate::{
    backends::BackendItem,
    conversion::{
        ConversionLog, ConversionResult, dropped_annotations_warning, dropped_feature_warning,
    },
    convert::{ConversionConfig, IdentifierKind, rename_identifier},
    ir::{LanguageBase, LanguageField, LanguageStruct, LanguageStructKind, Visibility},
    type_map::TargetLanguage,
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

    fn to_ir(self, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
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

        let method_options = PythonFunctionConversionOptions {
            config: options.map(|o| o.config.clone()).unwrap_or_default(),
        };
        let mut methods = Vec::with_capacity(self.methods.len());
        for method in self.methods {
            let result = method.to_ir(Some(&method_options));
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

        let mut ir = LanguageStruct {
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
        };
        if let Some(hooks) = options.and_then(|options| options.config.hooks.as_ref()) {
            hooks.after_to_ir_struct(&mut ir);
        }

        ConversionResult::with_log(ir, log)
    }

    fn from_ir(
        mut input: Self::IrType,
        options: Option<&Self::ConversionOptions>,
    ) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();
        let config = options
            .map(|options| options.config.clone())
            .unwrap_or_default();
        if let Some(hooks) = &config.hooks {
            hooks.before_from_ir_struct(&mut input);
        }

        let name = rename_identifier(
            &config,
            &input.name,
            TargetLanguage::Python,
            IdentifierKind::Type,
        );
        log.add_warnings(name.log.warnings);

        if !input.generic_args.is_empty() {
            log.add_warning(dropped_feature_warning(
                "generic arguments",
                &input.name,
                "Python",
            ));
        }

        if !input.annotations.is_empty() || !input.raw_attributes.is_empty() {
            log.add_warning(dropped_annotations_warning(
                input.annotations.len() + input.raw_attributes.len(),
                "class",
                &input.name,
                "Python",
            ));
        }

        if !input.fields.is_empty() {
            log.add_warning(dropped_feature_warning(
                "field values",
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
            fields.push(PythonClassField {
                name: field_name.value,
                value: "None".to_string(),
            });
        }

        let function_options = PythonFunctionConversionOptions {
            config: config.clone(),
        };
        let mut methods = Vec::with_capacity(input.methods.len());
        for method in input.methods {
            let result = PythonFunction::from_ir(method, Some(&function_options));
            log.add_warnings(result.log.warnings);
            methods.push(result.value);
        }

        let bases = input.bases.into_iter().map(|base| base.name).collect();

        ConversionResult::with_log(
            PythonClass {
                name: name.value,
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
    pub const DEFAULT: Self = Self {
        render_docstring: true,
    };
}
