#[derive(Debug)]
pub enum ErrorKind {
    FieldCount { want: usize, got: usize },
    IO(std::io::Error),
    UnclosedQuote,
    InvalidByte(u8),
    DoubleQuoteInUnescapedField,
    NonCommaAfterQuote(u8),
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    pub record: Option<usize>,
    pub field: Option<usize>,
    pub field_name: Option<String>,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Error {
            kind,
            record: None,
            field: None,
            field_name: None,
        }
    }
    pub fn record_num(mut self, record_num: usize) -> Self {
        self.record = Some(record_num);
        self
    }
    pub fn field_num(mut self, field_num: usize) -> Self {
        self.field = Some(field_num);
        self
    }

    pub fn field_name(mut self, field_name: String) -> Self {
        self.field_name = Some(field_name);
        self
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::new(ErrorKind::IO(value))
    }
}

impl From<ErrorKind> for Error {
    fn from(value: ErrorKind) -> Self {
        Error::new(value)
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.record, self.field, &self.field_name) {
            (None, None, None) => write!(f, "{}", self.kind),
            (None, Some(field), None) => write!(f, "field `{}`: {}", field + 1, self.kind),
            (None, _, Some(field)) => write!(f, "field `{}`: {}", field, self.kind),
            (Some(record), None, None) => write!(f, "record `{}`: {}", record, self.kind),
            (Some(record), Some(field), None) => {
                write!(
                    f,
                    "record `{}`: field: `{}`: {}",
                    record,
                    field + 1,
                    self.kind
                )
            }
            (Some(record), _, Some(field)) => {
                write!(f, "record `{}`: field: `{}`: {}", record, field, self.kind)
            }
        }
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ErrorKind::*;
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
            NonCommaAfterQuote(c) => write!(
                f,
                "expected comma or newline after double quote but found `{}`",
                *c as char
            ),
        }
    }
}
