use std::fmt;

use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    ir::Visibility,
};

/// Represents a Rust visibility specifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustVisibility {
    /// No specifier — private to the current module.
    Private,
    /// `pub` — visible everywhere.
    Pub,
    /// `pub(crate)` — visible within the crate.
    PubCrate,
    /// `pub(super)` — visible within the parent module.
    PubSuper,
}

impl fmt::Display for RustVisibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RustVisibility::Private => write!(f, ""),
            RustVisibility::Pub => write!(f, "pub"),
            RustVisibility::PubCrate => write!(f, "pub(crate)"),
            RustVisibility::PubSuper => write!(f, "pub(super)"),
        }
    }
}

impl RustVisibility {
    /// Returns the specifier followed by a single space, or an empty string when private.
    pub fn prefix(&self) -> String {
        match self {
            RustVisibility::Private => String::new(),
            other => format!("{} ", other),
        }
    }
}

impl BackendItem for RustVisibility {
    type IrType = Visibility;
    type ConversionOptions = RustVisibilityConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let ir = match self {
            RustVisibility::Private => Visibility::Private,
            RustVisibility::Pub => Visibility::Public,
            RustVisibility::PubCrate => Visibility::Package,
            RustVisibility::PubSuper => Visibility::ParentModule,
        };
        ConversionResult::new(ir)
    }

    fn from_ir(
        input: Self::IrType,
        _options: Option<&Self::ConversionOptions>,
    ) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();
        let value = match input {
            Visibility::Default | Visibility::Private => RustVisibility::Private,
            Visibility::Public => RustVisibility::Pub,
            Visibility::Package => RustVisibility::PubCrate,
            Visibility::ParentModule => RustVisibility::PubSuper,
            Visibility::Protected => {
                log.add_warning(ConversionWarning::VisibilityApproximated {
                    original: "Protected".to_string(),
                    approximated: "pub".to_string(),
                });
                RustVisibility::Pub
            }
            Visibility::Namespace => {
                log.add_warning(ConversionWarning::VisibilityApproximated {
                    original: "Namespace".to_string(),
                    approximated: "pub(crate)".to_string(),
                });
                RustVisibility::PubCrate
            }
            Visibility::Scoped(scope) => {
                log.add_warning(ConversionWarning::VisibilityApproximated {
                    original: format!("Scoped({})", scope),
                    approximated: "pub(crate)".to_string(),
                });
                RustVisibility::PubCrate
            }
        };
        ConversionResult::with_log(value, log)
    }
}

/// Conversion options for Rust visibility.
#[derive(Debug, Clone)]
pub struct RustVisibilityConversionOptions {}

impl Default for RustVisibilityConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl RustVisibilityConversionOptions {
    pub const DEFAULT: Self = Self {};
}
