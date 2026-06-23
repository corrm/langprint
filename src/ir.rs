//! Language-agnostic Intermediate Representation (IR) for declarations.
//!
//! The IR models the declaration concepts shared across target languages (types, fields, enums,
//! function signatures, namespaces, constants, visibility). It is the common exchange layer used
//! when converting between language backends; single-language features live in the native backend
//! models and are reported via [`crate::conversion::ConversionWarning`] when projected to the IR.

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
