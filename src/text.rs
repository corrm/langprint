#[derive(Debug, Clone, Copy)]
pub enum IndentStyle {
    Tabs,
    Spaces,
}

#[derive(Debug, Clone, Copy)]
pub enum NewLineStyle {
    CR,
    LF,
    CRLF,
}

impl NewLineStyle {
    pub const fn as_str(&self) -> &'static str {
        match self {
            NewLineStyle::CR => "\r",
            NewLineStyle::LF => "\n",
            NewLineStyle::CRLF => "\r\n",
        }
    }

    pub const fn len(&self) -> usize {
        match self {
            NewLineStyle::CR => 1,
            NewLineStyle::LF => 1,
            NewLineStyle::CRLF => 2,
        }
    }

    /// Returns `false` since all newline styles have non-zero length.
    pub const fn is_empty(&self) -> bool {
        false
    }
}
