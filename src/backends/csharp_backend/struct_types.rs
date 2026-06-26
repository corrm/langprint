use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    convert::{ConversionConfig, IdentifierKind, rename_identifier},
    ir::{
        LanguageBase, LanguageField, LanguageStruct, LanguageStructKind, RawAttribute, Visibility,
    },
    type_map::TargetLanguage,
};

use super::attributes::csharp_attribute_to_annotation;
use super::{
    CSharpField, CSharpFieldConversionOptions, CSharpGenericArgument, CSharpMethod,
    CSharpMethodConversionOptions, CSharpProperty, CSharpVisibility,
};

/// The kind of a C# type declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CSharpTypeKind {
    /// A `class`.
    Class,
    /// A `struct`.
    Struct,
    /// An `interface`.
    Interface,
    /// A `record`.
    Record,
}

impl CSharpTypeKind {
    /// The C# keyword for this kind.
    pub fn keyword(&self) -> &'static str {
        match self {
            CSharpTypeKind::Class => "class",
            CSharpTypeKind::Struct => "struct",
            CSharpTypeKind::Interface => "interface",
            CSharpTypeKind::Record => "record",
        }
    }

    /// Whether the `sealed` modifier is valid for this kind. Structs are implicitly
    /// sealed and interfaces cannot be sealed in C#, so `sealed` only applies to
    /// `class` and `record`.
    pub fn can_be_sealed(&self) -> bool {
        matches!(self, CSharpTypeKind::Class | CSharpTypeKind::Record)
    }

    /// Whether the `abstract` modifier is valid for this kind. Structs are sealed value
    /// types and interfaces are implicitly abstract, so `abstract` only applies to
    /// `class` and `record`.
    pub fn can_be_abstract(&self) -> bool {
        matches!(self, CSharpTypeKind::Class | CSharpTypeKind::Record)
    }

    /// Whether the `unsafe` modifier is valid for this kind. polyplugc keeps structs safe
    /// by design, so `unsafe` applies to every kind except `struct`.
    pub fn can_be_unsafe(&self) -> bool {
        !matches!(self, CSharpTypeKind::Struct)
    }
}

/// Represents a C# type declaration (class / struct / interface / record).
#[derive(Debug, Clone, PartialEq)]
pub struct CSharpType {
    /// The kind of the type.
    pub kind: CSharpTypeKind,
    /// The name of the type.
    pub name: String,
    /// The visibility of the type.
    pub visibility: CSharpVisibility,
    /// Whether the type is `abstract`.
    pub is_abstract: bool,
    /// Whether the type is `sealed`.
    pub is_sealed: bool,
    /// Whether the type is `static`.
    pub is_static: bool,
    /// Whether the type is `unsafe`. Never valid on a `struct`; see [`CSharpTypeKind::can_be_unsafe`].
    pub is_unsafe: bool,
    /// Whether the type is `partial`.
    pub is_partial: bool,
    /// Generic type parameters.
    pub generic_args: Vec<CSharpGenericArgument>,
    /// The base class, if any.
    pub base_class: Option<String>,
    /// Implemented interfaces.
    pub interfaces: Vec<String>,
    /// The fields of the type.
    pub fields: Vec<CSharpField>,
    /// The properties of the type.
    pub properties: Vec<CSharpProperty>,
    /// The methods of the type.
    pub methods: Vec<CSharpMethod>,
    /// Attributes applied to the type (without the leading `[`).
    pub attributes: Vec<String>,
    /// Documentation for the type.
    pub docs: Option<Vec<String>>,
}

impl BackendItem for CSharpType {
    type IrType = LanguageStruct;
    type ConversionOptions = CSharpTypeConversionOptions;

    fn to_ir(self, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut log = ConversionLog::new();

        let struct_kind = match self.kind {
            CSharpTypeKind::Struct => LanguageStructKind::Struct,
            CSharpTypeKind::Class => LanguageStructKind::Class,
            CSharpTypeKind::Interface => {
                log.add_warning(ConversionWarning::UnsupportedFeature {
                    feature: format!("`interface` kind on type `{}`", self.name),
                    resolution: "interface flattened to a class in the language-agnostic IR"
                        .to_string(),
                });
                LanguageStructKind::Class
            }
            CSharpTypeKind::Record => {
                log.add_warning(ConversionWarning::UnsupportedFeature {
                    feature: format!("`record` kind on type `{}`", self.name),
                    resolution: "record flattened to a class in the language-agnostic IR"
                        .to_string(),
                });
                LanguageStructKind::Class
            }
        };

        if self.is_static {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("`static` on type `{}`", self.name),
                resolution: "static modifier dropped from the language-agnostic IR".to_string(),
            });
        }
        if self.is_partial {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("`partial` on type `{}`", self.name),
                resolution: "partial modifier dropped from the language-agnostic IR".to_string(),
            });
        }
        if self.is_unsafe {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("`unsafe` on type `{}`", self.name),
                resolution: "unsafe modifier dropped from the language-agnostic IR".to_string(),
            });
        }
        let mut annotations = Vec::new();
        let mut raw_attributes = Vec::new();
        for attribute in &self.attributes {
            match csharp_attribute_to_annotation(attribute) {
                Some(annotation) => annotations.push(annotation),
                None => raw_attributes.push(RawAttribute {
                    source: TargetLanguage::CSharp,
                    text: attribute.clone(),
                }),
            }
        }

        let visibility = self.visibility.to_ir(None);
        log.add_warnings(visibility.log.warnings);

        let mut bases = Vec::new();
        if let Some(base_class) = self.base_class {
            bases.push(LanguageBase {
                name: base_class,
                visibility: Visibility::Public,
            });
        }
        for interface in self.interfaces {
            bases.push(LanguageBase {
                name: interface,
                visibility: Visibility::Public,
            });
        }

        let mut fields = Vec::with_capacity(self.fields.len() + self.properties.len());
        for field in self.fields {
            let result = field.to_ir(None);
            log.add_warnings(result.log.warnings);
            fields.push(result.value);
        }
        for property in self.properties {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("property `{}` on type `{}`", property.name, self.name),
                resolution: "property lowered to a field in the language-agnostic IR".to_string(),
            });
            let visibility = property.visibility.to_ir(None);
            log.add_warnings(visibility.log.warnings);
            fields.push(LanguageField {
                name: property.name,
                field_type: property.prop_type,
                visibility: visibility.value,
                is_static: property.is_static,
                is_const: false,
                docs: property.docs,
                annotations: Vec::new(),
                raw_attributes: Vec::new(),
            });
        }

        let mut methods = Vec::with_capacity(self.methods.len());
        for method in self.methods {
            let result = method.to_ir(None);
            log.add_warnings(result.log.warnings);
            methods.push(result.value);
        }

        let mut generic_args = Vec::with_capacity(self.generic_args.len());
        for generic in self.generic_args {
            let result = generic.to_ir(None);
            log.add_warnings(result.log.warnings);
            generic_args.push(result.value);
        }

        let mut language_struct = LanguageStruct {
            visibility: visibility.value,
            struct_kind,
            is_abstract: self.is_abstract,
            is_final: self.is_sealed,
            name: self.name,
            generic_args,
            bases,
            fields,
            methods,
            docs: self.docs,
            annotations,
            raw_attributes,
        };
        if let Some(hooks) = options.and_then(|options| options.config.hooks.as_ref()) {
            hooks.after_to_ir_struct(&mut language_struct);
        }
        ConversionResult::with_log(language_struct, log)
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

        let mut kind = match input.struct_kind {
            LanguageStructKind::Class => CSharpTypeKind::Class,
            LanguageStructKind::Struct => CSharpTypeKind::Struct,
            LanguageStructKind::Union => {
                log.add_warning(ConversionWarning::UnsupportedFeature {
                    feature: format!("union `{}`", input.name),
                    resolution: "C# has no union; rendered as a struct".to_string(),
                });
                CSharpTypeKind::Struct
            }
        };

        // C# value types cannot be abstract, so an abstract IR struct/union is lowered to a class.
        if input.is_abstract && !kind.can_be_abstract() {
            log.add_warning(ConversionWarning::UnsupportedFeature {
                feature: format!("abstract value type `{}`", input.name),
                resolution: "C# structs cannot be abstract; lowered to a class".to_string(),
            });
            kind = CSharpTypeKind::Class;
        }

        let name = rename_identifier(
            &config,
            &input.name,
            TargetLanguage::CSharp,
            IdentifierKind::Type,
        );
        log.add_warnings(name.log.warnings);

        let visibility = CSharpVisibility::from_ir(input.visibility, None);
        log.add_warnings(visibility.log.warnings);

        let mut attributes = Vec::new();
        for annotation in &input.annotations {
            if let Some(rendered) = config
                .annotation_map
                .resolve(TargetLanguage::CSharp, annotation)
            {
                attributes.push(rendered);
            }
        }
        for raw in &input.raw_attributes {
            if raw.source != TargetLanguage::CSharp {
                log.add_warning(ConversionWarning::UnsupportedFeature {
                    feature: format!("opaque {:?} attribute `{}`", raw.source, raw.text),
                    resolution: "cannot translate to C#; dropped".to_string(),
                });
                continue;
            }
            attributes.push(raw.text.clone());
        }

        let mut bases = input.bases.into_iter();
        let base_class = bases.next().map(|base| base.name);
        let interfaces = bases.map(|base| base.name).collect();

        let field_options = CSharpFieldConversionOptions {
            config: config.clone(),
        };
        let mut fields = Vec::with_capacity(input.fields.len());
        for field in input.fields {
            let result = CSharpField::from_ir(field, Some(&field_options));
            log.add_warnings(result.log.warnings);
            fields.push(result.value);
        }

        let method_options = CSharpMethodConversionOptions {
            config: config.clone(),
        };
        let mut methods = Vec::with_capacity(input.methods.len());
        for method in input.methods {
            let result = CSharpMethod::from_ir(method, Some(&method_options));
            log.add_warnings(result.log.warnings);
            methods.push(result.value);
        }

        let mut generic_args = Vec::with_capacity(input.generic_args.len());
        for generic in &input.generic_args {
            let result = CSharpGenericArgument::from_ir(generic.clone(), None);
            log.add_warnings(result.log.warnings);
            generic_args.push(result.value);
        }

        let csharp_type = CSharpType {
            kind,
            name: name.value,
            visibility: visibility.value,
            is_abstract: input.is_abstract,
            is_sealed: input.is_final,
            is_static: false,
            is_unsafe: false,
            is_partial: false,
            generic_args,
            base_class,
            interfaces,
            fields,
            properties: Vec::new(),
            methods,
            attributes,
            docs: input.docs,
        };
        ConversionResult::with_log(csharp_type, log)
    }
}

/// Conversion options for C# types.
#[derive(Debug, Clone, Default)]
pub struct CSharpTypeConversionOptions {
    /// Cross-language conversion configuration (type mapping + renaming).
    pub config: ConversionConfig,
}

/// Render options for C# types.
#[derive(Debug, Clone)]
pub struct CSharpTypeRenderOptions {
    /// Whether to render documentation comments.
    pub render_docs: bool,
    /// Whether to render attributes.
    pub render_attributes: bool,
    /// Whether to render properties.
    pub render_properties: bool,
    /// Whether to render methods.
    pub render_methods: bool,
}

impl Default for CSharpTypeRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CSharpTypeRenderOptions {
    pub const DEFAULT: Self = Self {
        render_docs: true,
        render_attributes: true,
        render_properties: true,
        render_methods: true,
    };
}
