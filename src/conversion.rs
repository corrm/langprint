/// Types of warnings that can occur during conversion.
#[derive(Debug, Clone, PartialEq)]
pub enum ConversionWarning {
    /// Feature not supported in target language.
    UnsupportedFeature {
        /// Description of the unsupported feature.
        feature: String,
        /// What was done to handle the unsupported feature.
        resolution: String,
    },
    /// Visibility level approximated.
    VisibilityApproximated {
        /// Original visibility.
        original: String,
        /// Approximated visibility.
        approximated: String,
    },
    /// Naming convention changed.
    NamingConventionChanged {
        /// Original name.
        original: String,
        /// Converted name.
        converted: String,
    },
    /// Other warning with a description.
    Other(String),
}

/// Log of conversion warnings.
#[derive(Debug, Clone, Default)]
pub struct ConversionLog {
    /// List of warnings generated during conversion.
    pub warnings: Vec<ConversionWarning>,
}

impl ConversionLog {
    /// Create a new empty conversion log.
    pub fn new() -> Self {
        Self { warnings: Vec::new() }
    }
}

impl ConversionLog {
    /// Add a warning to the log.
    pub fn add_warning(&mut self, warning: ConversionWarning) -> &mut Self {
        self.warnings.push(warning);
        self
    }

    /// Add multiple warnings to the log.
    pub fn add_warnings(&mut self, warnings: impl IntoIterator<Item = ConversionWarning>) -> &mut Self {
        self.warnings.extend(warnings);
        self
    }

    /// Check if the log has any warnings.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// Result of a conversion operation, including the converted value and any warnings.
#[derive(Debug, Clone)]
pub struct ConversionResult<T> {
    /// The converted value.
    pub value: T,
    /// Log of warnings generated during conversion.
    pub log: ConversionLog,
}

impl<T> ConversionResult<T> {
    /// Create a new conversion result with a value and empty log.
    pub fn new(value: T) -> Self {
        Self {
            value,
            log: ConversionLog::new(),
        }
    }

    /// Create a new conversion result with a value and existing log.
    pub fn with_log(value: T, log: ConversionLog) -> Self {
        Self { value, log }
    }

    /// Create a new conversion result with a value and a single warning.
    pub fn with_warning(value: T, warning: ConversionWarning) -> Self {
        let mut log = ConversionLog::new();
        log.add_warning(warning);

        Self { value, log }
    }
}

impl<T> ConversionResult<T> {
    /// Add a warning to the conversion result.
    pub fn add_warning(&mut self, warning: ConversionWarning) -> &mut Self {
        self.log.add_warning(warning);
        self
    }

    /// Map the value inside the conversion result.
    pub fn map<U, F>(self, f: F) -> ConversionResult<U>
    where
        F: FnOnce(T) -> U,
    {
        ConversionResult {
            value: f(self.value),
            log: self.log,
        }
    }
}
