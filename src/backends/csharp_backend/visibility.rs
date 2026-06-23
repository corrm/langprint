use std::fmt;

use crate::{
    backends::BackendItem,
    conversion::{ConversionLog, ConversionResult, ConversionWarning},
    ir::Visibility,
};

/// Represents a C# access modifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CSharpVisibility {
    /// No explicit modifier — defaults to the member's implicit access.
    Default,
    /// `public`.
    Public,
    /// `private`.
    Private,
    /// `protected`.
    Protected,
    /// `internal`.
    Internal,
    /// `protected internal`.
    ProtectedInternal,
    /// `private protected`.
    PrivateProtected,
}

impl fmt::Display for CSharpVisibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CSharpVisibility::Default => write!(f, ""),
            CSharpVisibility::Public => write!(f, "public"),
            CSharpVisibility::Private => write!(f, "private"),
            CSharpVisibility::Protected => write!(f, "protected"),
            CSharpVisibility::Internal => write!(f, "internal"),
            CSharpVisibility::ProtectedInternal => write!(f, "protected internal"),
            CSharpVisibility::PrivateProtected => write!(f, "private protected"),
        }
    }
}

impl CSharpVisibility {
    /// Returns the modifier followed by a single space, or an empty string when [`CSharpVisibility::Default`].
    pub fn prefix(&self) -> String {
        match self {
            CSharpVisibility::Default => String::new(),
            other => format!("{} ", other),
        }
    }
}

impl BackendItem for CSharpVisibility {
    type IrType = Visibility;
    type ConversionOptions = CSharpVisibilityConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        let mut log = ConversionLog::new();
        let ir = match self {
            CSharpVisibility::Default => Visibility::Default,
            CSharpVisibility::Public => Visibility::Public,
            CSharpVisibility::Private => Visibility::Private,
            CSharpVisibility::Protected => Visibility::Protected,
            CSharpVisibility::Internal => Visibility::Package,
            CSharpVisibility::ProtectedInternal => {
                log.add_warning(ConversionWarning::VisibilityApproximated {
                    original: "protected internal".to_string(),
                    approximated: "Protected".to_string(),
                });
                Visibility::Protected
            }
            CSharpVisibility::PrivateProtected => {
                log.add_warning(ConversionWarning::VisibilityApproximated {
                    original: "private protected".to_string(),
                    approximated: "Protected".to_string(),
                });
                Visibility::Protected
            }
        };
        ConversionResult::with_log(ir, log)
    }

    fn from_ir(input: Self::IrType, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self> {
        let mut log = ConversionLog::new();
        let value = match input {
            Visibility::Default => CSharpVisibility::Default,
            Visibility::Public => CSharpVisibility::Public,
            Visibility::Private => CSharpVisibility::Private,
            Visibility::Protected => CSharpVisibility::Protected,
            Visibility::Package | Visibility::Namespace => CSharpVisibility::Internal,
            Visibility::ParentModule => {
                log.add_warning(ConversionWarning::VisibilityApproximated {
                    original: "ParentModule".to_string(),
                    approximated: "internal".to_string(),
                });
                CSharpVisibility::Internal
            }
            Visibility::Scoped(scope) => {
                log.add_warning(ConversionWarning::VisibilityApproximated {
                    original: format!("Scoped({})", scope),
                    approximated: "internal".to_string(),
                });
                CSharpVisibility::Internal
            }
        };
        ConversionResult::with_log(value, log)
    }
}

/// Conversion options for C# visibility.
#[derive(Debug, Clone)]
pub struct CSharpVisibilityConversionOptions {}

impl Default for CSharpVisibilityConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CSharpVisibilityConversionOptions {
    pub const DEFAULT: Self = Self {};
}
