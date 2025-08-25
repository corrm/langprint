use std::fmt::Display;

use crate::{
    backends::BackendItem,
    conversion::{ConversionResult, ConversionWarning},
    ir::Visibility,
};

/// C++ visibility modifiers for classes, structs, and enums.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CppVisibility {
    /// Default access (no explicit specifier).
    Default,
    /// Public access.
    Public,
    /// Protected access (only relevant within classes).
    Protected,
    /// Private access.
    Private,
}

impl Display for CppVisibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CppVisibility::Public => write!(f, "public"),
            CppVisibility::Protected => write!(f, "protected"),
            CppVisibility::Private => write!(f, "private"),
            CppVisibility::Default => write!(f, ""),
        }
    }
}

impl BackendItem for CppVisibility {
    type IrType = Visibility;
    type ConversionOptions = CppVisibilityConversionOptions;

    fn to_ir(self, _options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType> {
        match self {
            CppVisibility::Public => ConversionResult::new(Visibility::Public),
            CppVisibility::Protected => ConversionResult::new(Visibility::Protected),
            CppVisibility::Private => ConversionResult::new(Visibility::Private),
            CppVisibility::Default => ConversionResult::new(Visibility::Default),
        }
    }

    fn from_ir(
        input: Self::IrType,
        _options: Option<&Self::ConversionOptions>,
    ) -> ConversionResult<Self> {
        match input {
            Visibility::Default => ConversionResult::new(CppVisibility::Default),
            Visibility::Public => ConversionResult::new(CppVisibility::Public),
            Visibility::Private => ConversionResult::new(CppVisibility::Private),
            Visibility::Protected => ConversionResult::new(CppVisibility::Protected),
            Visibility::Package => ConversionResult::with_warning(
                CppVisibility::Default,
                ConversionWarning::VisibilityApproximated {
                    original: "package".to_string(),
                    approximated: "default".to_string(),
                },
            ),
            Visibility::Namespace => ConversionResult::with_warning(
                CppVisibility::Public,
                ConversionWarning::VisibilityApproximated {
                    original: "namespace".to_string(),
                    approximated: "public in namespace".to_string(),
                },
            ),
            Visibility::Scoped(scope) => {
                // C++ handles scoping via namespaces
                ConversionResult::with_warning(
                    CppVisibility::Public,
                    ConversionWarning::VisibilityApproximated {
                        original: format!("scoped to {}", scope),
                        approximated: "public in namespace".to_string(),
                    },
                )
            }
            Visibility::ParentModule => ConversionResult::with_warning(
                CppVisibility::Default,
                ConversionWarning::VisibilityApproximated {
                    original: "parent module".to_string(),
                    approximated: "default".to_string(),
                },
            ),
        }
    }
}

/// Conversion options for C++ visibility.
#[derive(Debug, Clone)]
pub struct CppVisibilityConversionOptions {}

impl Default for CppVisibilityConversionOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CppVisibilityConversionOptions {
    pub const DEFAULT: Self = Self {};
}

/// Render options for C++ visibility.
#[derive(Debug, Clone)]
pub struct CppVisibilityRenderOptions {}

impl Default for CppVisibilityRenderOptions {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl CppVisibilityRenderOptions {
    pub const DEFAULT: Self = Self {};
}
