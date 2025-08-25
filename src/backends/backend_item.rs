use crate::conversion::ConversionResult;

pub trait BackendItem: Sized {
    /// The intermediate representation type this item is converted to/from.
    type IrType;
    /// The conversion options type.
    type ConversionOptions: Default;

    /// Convert from the language-specific type to the intermediate representation.
    fn to_ir(self, options: Option<&Self::ConversionOptions>) -> ConversionResult<Self::IrType>;
    /// Convert from the intermediate representation to the language-specific type.
    fn from_ir(
        input: Self::IrType,
        options: Option<&Self::ConversionOptions>,
    ) -> ConversionResult<Self>;
}
