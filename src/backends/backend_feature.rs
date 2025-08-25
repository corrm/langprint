/// Features supported by language backends.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackendFeature {
    /// Support for define values.
    Define,
    /// Support for namespace/package/module.
    Namespace,
    /// Support for constant values.
    Constant,
    /// Support for function types.
    Function,
    /// Support for enum types.
    Enum,
    /// Support for struct types.
    Struct,
    /// Support for class types.
    Class,
    /// Support for interface types.
    Interface,
}
