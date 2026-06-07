#[derive(Debug)]
pub enum Error {
    FieldCount { want: usize, got: usize },
    IO(std::io::Error),
    UnclosedQuote,
    InvalidByte(u8),
    DoubleQuoteInUnescapedField,
    NonCommaAfterQuote(u8),
    LineError(usize, Box<Error>),
    FieldError(usize, Box<Error>),
}

impl Error {
    pub fn add_line(self, line: usize) -> Self {
        Error::LineError(line, Box::new(self))
    }

    pub fn add_field(self, field: usize) -> Self {
        Error::FieldError(field, Box::new(self))
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IO(value)
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            FieldCount { want, got } => write!(f, "expected `{want}` fields but found `{got}`"),
            IO(e) => write!(f, "{e}"),
            UnclosedQuote => write!(f, "unclosed quote"),
            InvalidByte(b) => {
                if b.is_ascii() {
                    write!(f, "invalid character `{}`", *b as char)
                } else {
                    write!(f, "unexpected byte `0x{:X}`", *b)
                }
            }
            DoubleQuoteInUnescapedField => {
                write!(f, "double quote in unescaped field")
            }
            LineError(line, e) => write!(f, "line {line}: {e}"),
            FieldError(field, e) => write!(f, "field {field}: {e}"),
            NonCommaAfterQuote(c) => write!(
                f,
                "expected comma or newline after double quote but found `{}`",
                *c as char
            ),
        }
    }
}
