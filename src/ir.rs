//! Intermediate Representation (IR) module for language-agnostic enum representation.

mod language_constant;
mod language_definition;
mod language_enum;
mod language_field;
mod language_function;
mod language_generic_argument;
mod language_namespace;
mod language_struct;
mod visibility;

pub use language_constant::LanguageConstant;
pub use language_definition::LanguageDefinition;
pub use language_enum::{EnumVariant, EnumVariantValue, LanguageEnum};
pub use language_field::LanguageField;
pub use language_function::{LanguageFunction, LanguageFunctionParameter};
pub use language_generic_argument::LanguageGenericArgument;
pub use language_namespace::LanguageNamespace;
pub use language_struct::{LanguageBase, LanguageStruct, LanguageStructKind};
pub use visibility::Visibility;
