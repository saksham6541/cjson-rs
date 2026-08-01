#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    Parse,
    TypeMismatch,
    MissingKey,
    OutOfBounds,
    InvalidOperation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub message: String,
    pub position: usize,
}

impl ParseError {
    pub fn new(message: impl Into<String>, position: usize) -> Self {
        Self {
            kind: ParseErrorKind::Parse,
            message: message.into(),
            position,
        }
    }

    pub fn type_mismatch(message: impl Into<String>) -> Self {
        Self {
            kind: ParseErrorKind::TypeMismatch,
            message: message.into(),
            position: 0,
        }
    }

    pub fn missing_key(message: impl Into<String>) -> Self {
        Self {
            kind: ParseErrorKind::MissingKey,
            message: message.into(),
            position: 0,
        }
    }

    pub fn out_of_bounds(message: impl Into<String>, index: usize) -> Self {
        Self {
            kind: ParseErrorKind::OutOfBounds,
            message: message.into(),
            position: index,
        }
    }

    pub fn invalid_operation(message: impl Into<String>) -> Self {
        Self {
            kind: ParseErrorKind::InvalidOperation,
            message: message.into(),
            position: 0,
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.position > 0 {
            write!(f, "{} at byte {}", self.message, self.position)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for ParseError {}
