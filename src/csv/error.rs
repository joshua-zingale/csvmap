#[derive(Debug)]
pub enum Error {
    FieldCount { want: usize, got: usize },
    IO(std::io::Error),
    UnclosedQuote,
    InvalidByte(u8),
    DoubleQuoteInUnescapedField,
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
            IO(e) => e.fmt(f),
            UnclosedQuote => write!(f, "unclosed quote"),
            InvalidByte(b) => {
                if b.is_ascii() {
                    write!(f, "invalid character `{}`", b.to_string())
                } else {
                    write!(f, "unexpected byte `0x{:X}`", *b)
                }
            }
            DoubleQuoteInUnescapedField => {
                write!(f, "double quote in unescaped field")
            }
        }
    }
}
