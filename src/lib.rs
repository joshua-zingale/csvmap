use std::io::BufRead;

pub struct File<R: std::io::Read> {
    reader: std::io::BufReader<R>,
    buffer: Vec<u8>,
    first: bool,
}

impl<T: std::io::Read> File<T> {
    pub fn new(reader: T) -> Self {
        File {
            reader: std::io::BufReader::new(reader),
            buffer: Vec::new(),
            first: true,
        }
    }

    pub fn read<'a>(&'a mut self) -> Result<Option<Record<'a>>, std::io::Error> {
        self.buffer.clear();
        let first = self.first;
        self.first = false;

        let mut num_quotes = 0;
        loop {
            let last_size = self.buffer.len();
            let num_read = self.reader.read_until(b'\n', &mut self.buffer)?;
            if !first && num_read == 0 {
                return Ok(None);
            }
            num_quotes += self.buffer[last_size..]
                .iter()
                .filter(|&&v| v == b'"')
                .count();

            if num_quotes % 2 == 0 {
                if self.buffer.ends_with(b"\n") {
                    self.buffer.pop();
                    if self.buffer.ends_with(b"\r") {
                        self.buffer.pop();
                    }
                }
                return Ok(Some(Record(&self.buffer)));
            } else if self.buffer.len() == last_size {
                return Ok(Some(Record(&self.buffer)));
            }
        }
    }
}

#[derive(Clone)]
pub struct Record<'a>(&'a [u8]);

impl<'a> Record<'a> {
    pub fn iter(&self) -> RecordIter<'a> {
        RecordIter {
            rest: self.0,
            is_next: true,
        }
    }
}

pub struct RecordIter<'a> {
    rest: &'a [u8],
    is_next: bool,
}

impl<'a> Iterator for RecordIter<'a> {
    type Item = Result<Field<'a>, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_next {
            return None;
        }

        if self.rest.starts_with(b"\"") {
            let mut parts = Vec::new();

            let mut rest = &self.rest[1..];

            loop {
                if let Some(quote_pos) = rest.iter().position(|&c| c == b'"') {
                    println!("'{}'", String::from_utf8_lossy(rest));
                    match rest.get(quote_pos + 1) {
                        None => {
                            self.is_next = false;
                            parts.push(&rest[..quote_pos]);
                            break;
                        }
                        Some(b',') => {
                            parts.push(&rest[..quote_pos]);
                            self.rest = &rest[quote_pos + 2..];
                            break;
                        }
                        Some(b'"') => {
                            parts.push(&rest[..=quote_pos]);
                            rest = &rest[quote_pos + 2..];
                        }
                        Some(&c) => {
                            return Some(Err(format!("unexpected byte `0x{:X}`", c)));
                        }
                    }
                } else {
                    return Some(Err("unclosed double quote".to_string()));
                }
            }

            Some(Ok(Field(_Field::Escaped(parts))))
        } else {
            let comma_pos = self.rest.iter().position(|&c| c == b',');
            let quote_pos = self.rest.iter().position(|&c| c == b'"');

            match (comma_pos, quote_pos) {
                (None, None) => {
                    self.is_next = false;
                    Some(Ok(Field(_Field::Clean(self.rest))))
                }
                (None, Some(_)) => {
                    self.is_next = false;
                    Some(Err("double quote in unescaped field".to_string()))
                }
                (Some(cp), Some(qp)) if qp < cp => {
                    Some(Err("double quote in unescaped field".to_string()))
                }
                (Some(cp), _) => {
                    let field = _Field::Clean(&self.rest[..cp]);
                    self.rest = &self.rest[cp + 1..];
                    Some(Ok(Field(field)))
                }
            }
        }
    }
}

pub struct Field<'a>(_Field<'a>);

impl<'a> std::fmt::Display for Field<'a> {
    /// Makes a lossy conversion in the event that the underlying data
    /// are invalid utf8.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            _Field::Clean(s) => write!(f, "{}", String::from_utf8_lossy(s)),
            _Field::Escaped(parts) => {
                for part in parts {
                    write!(f, "{}", String::from_utf8_lossy(part))?;
                }
                Ok(())
            }
        }
    }
}

impl<'a> Field<'a> {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        self.write_to(&mut buffer)
            .expect("writing to Vec should never fail");
        buffer
    }
    pub fn write_to<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        match &self.0 {
            _Field::Clean(s) => writer.write_all(s),
            _Field::Escaped(parts) => {
                for part in parts {
                    writer.write_all(part)?
                }
                Ok(())
            }
        }
    }
}

enum _Field<'a> {
    Clean(&'a [u8]),
    Escaped(EscapedIter<'a>),
}

type EscapedIter<'a> = Vec<&'a [u8]>;

#[cfg(test)]
impl<'a> File<&'a [u8]> {
    fn read_to_vec(&mut self) -> Vec<String> {
        self.read()
            .expect("Major I/O failure")
            .expect("Expected a row, found EOF")
            .iter()
            .map(|f| f.unwrap().to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_one_unquoted_row() {
        let mut f = File::new("a,bc , def".as_bytes());

        let vec = f.read_to_vec();

        assert_eq!(vec!["a", "bc ", " def"], vec);
    }

    #[test]
    fn reads_three_unquoted_rows() {
        let mut f = File::new("a,b\n1,2,3\n3".as_bytes());

        assert_eq!(vec!["a", "b"], f.read_to_vec());
        assert_eq!(vec!["1", "2", "3"], f.read_to_vec(),);
        assert_eq!(vec!["3"], f.read_to_vec(),);
        assert!(f.read().unwrap().is_none());
    }

    #[test]
    fn reads_empty_string_as_one_empty_field() {
        let mut f = File::new("".as_bytes());

        assert_eq!(vec![""], f.read_to_vec());
        assert!(f.read().unwrap().is_none());
    }

    #[test]
    fn reads_newline_as_one_empty_fiel() {
        let mut f = File::new("\n".as_bytes());

        assert_eq!(vec![""], f.read_to_vec());
        assert!(f.read().unwrap().is_none());

        let mut f = File::new("\r\n".as_bytes());

        assert_eq!(vec![""], f.read_to_vec());
        assert!(f.read().unwrap().is_none());
    }

    #[test]
    fn reads_one_single_empty_field_row_for_single_field() {
        let mut f = File::new(" Hello world! ".as_bytes());

        assert_eq!(vec![" Hello world! "], f.read_to_vec());
        assert!(f.read().unwrap().is_none());
    }

    #[test]
    fn reads_one_quoted_field() {
        let mut f = File::new("\" Hello world! \"".as_bytes());

        assert_eq!(vec![" Hello world! "], f.read_to_vec());
        assert!(f.read().unwrap().is_none());
    }

    #[test]
    fn reads_one_quoted_field_with_comma() {
        let mut f = File::new("\" Hello, world! \"".as_bytes());

        assert_eq!(vec![" Hello, world! "], f.read_to_vec(),);
        assert!(f.read().unwrap().is_none());
    }

    #[test]
    fn reads_single_character_quoted_field() {
        let mut f = File::new("\"a\"".as_bytes());

        assert_eq!(vec!["a"], f.read_to_vec());
        assert!(f.read().unwrap().is_none());
    }

    #[test]
    fn reads_comma_character_quoted_field() {
        let mut f = File::new("\",\"".as_bytes());

        assert_eq!(vec![","], f.read_to_vec());
        assert!(f.read().unwrap().is_none());
    }

    #[test]
    fn reads_quoted_field_with_newlines() {
        let mut f = File::new("\"i\nlike\r\nyou\"".as_bytes());
        assert_eq!(vec!["i\nlike\r\nyou"], f.read_to_vec());
        assert!(f.read().unwrap().is_none());
    }

    #[test]
    fn reads_double_quote_alone_in_quoted_field() {
        let mut f = File::new("\"\"\"\"".as_bytes());
        assert_eq!(vec!["\""], f.read_to_vec());
        assert!(f.read().unwrap().is_none());
    }

    #[test]
    fn error_quote_in_unquoted_field() {
        let mut f = File::new("data,\"bad_quote,more_data".as_bytes());
        let record = f.read().unwrap().expect("Should find a row");
        let mut iter = record.iter();

        assert!(iter.next().unwrap().is_ok());
        assert!(iter.next().unwrap().is_err());
    }

    #[test]
    fn error_stray_quote_in_quoted_field() {
        let mut f = File::new("\"The \"Great\" Gatsby\",1925".as_bytes());
        let record = f.read().unwrap().expect("Should find a row");
        let mut iter = record.iter();

        assert!(iter.next().unwrap().is_err());
    }

    #[test]
    fn error_incomplete_escape_at_end_of_field() {
        let mut f = File::new("\"starts but never ends".as_bytes());
        assert!(f.read().unwrap().unwrap().iter().next().unwrap().is_err());
    }

    #[test]
    fn error_garbage_after_closing_quote() {
        let mut f = File::new("\"field\"invalid_continuation,next".as_bytes());
        let record = f.read().unwrap().expect("Should find a row");
        let mut iter = record.iter();

        assert!(iter.next().unwrap().is_err());
    }

    #[test]
    fn error_imbalanced_quotes_multiline() {
        let mut f = File::new("\"line one\n line two \"\" still open\n".as_bytes());
        assert!(f.read().unwrap().unwrap().iter().next().unwrap().is_err());
    }
}
