use super::error::Error;
use std::io::BufRead;

/// Reads one CSV record `reader` into `buf`;
/// does not fail if the record is malformed.
///
/// Use the returned [Record] struct to determine
/// to determine if the record, i.e. any of its
/// fields, is malformed.
pub fn read_record<R: std::io::Read>(
    reader: &mut std::io::BufReader<R>,
    mut buf: &mut Vec<u8>,
) -> std::io::Result<usize> {
    let mut num_quotes = 0;
    let mut num_read = 0;
    loop {
        let last_size = buf.len();
        let chunk_size = reader.read_until(b'\n', &mut buf)?;
        num_read += chunk_size;

        if chunk_size == 0 {
            return Ok(num_read);
        }

        num_quotes += buf[last_size..].iter().filter(|&&v| v == b'"').count();

        if num_quotes % 2 == 0 {
            if buf.ends_with(b"\n") {
                buf.pop();
                if buf.ends_with(b"\r") {
                    buf.pop();
                }
            }
            return Ok(num_read);
        }
    }
}

/// A CSV Record.
#[derive(Clone)]
pub struct Record<'a>(&'a [u8]);

impl<'a> Record<'a> {
    pub fn new(record: &'a [u8]) -> Self {
        Record(record)
    }
    /// Reads `expected_fields` fields from this [Record] into `buf`.
    ///
    /// If `expected_fields` is `None`, any number of fields
    /// is accepted. Otherwise, this errors in the event of
    /// an incorrect number of fields being in this [Record].
    ///
    /// Halts on the first field that errors.
    pub fn read(
        &self,
        buf: &mut Vec<Field<'a>>,
        expected_fields: Option<usize>,
    ) -> Result<(), Error> {
        let mut fields_read = 0;
        for (i, field) in self.iter().enumerate() {
            if let Some(n) = expected_fields
                && i >= n
            {
                return Err(Error::FieldCount { want: n, got: i });
            }
            let field = field.map_err(|e| e.add_field(i))?;
            buf.push(field);
            fields_read += 1;
        }

        if let Some(n) = expected_fields
            && n > fields_read
        {
            return Err(Error::FieldCount {
                want: n,
                got: fields_read,
            });
        }

        Ok(())
    }
    pub fn iter(&self) -> RecordIter<'a> {
        RecordIter {
            rest: self.0,
            is_next: true,
        }
    }
}

/// Iterates over the fields in a Record.
pub struct RecordIter<'a> {
    rest: &'a [u8],
    is_next: bool,
}

impl<'a> Iterator for RecordIter<'a> {
    type Item = Result<Field<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_next {
            return None;
        }

        if self.rest.starts_with(b"\"") {
            let mut parts = Vec::new();

            let mut rest = &self.rest[1..];

            loop {
                if let Some(quote_pos) = rest.iter().position(|&c| c == b'"') {
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
                            return Some(Err(Error::InvalidByte(c)));
                        }
                    }
                } else {
                    return Some(Err(Error::UnclosedQuote));
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
                    Some(Err(Error::DoubleQuoteInUnescapedField))
                }
                (Some(cp), Some(qp)) if qp < cp => Some(Err(Error::DoubleQuoteInUnescapedField)),
                (Some(cp), _) => {
                    let field = _Field::Clean(&self.rest[..cp]);
                    self.rest = &self.rest[cp + 1..];
                    Some(Ok(Field(field)))
                }
            }
        }
    }
}

#[derive(Debug)]

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
    pub fn contents_eq(&self, other: &[u8]) -> bool {
        match &self.0 {
            _Field::Clean(s) => *s == other,
            _Field::Escaped(parts) => {
                let mut pos = 0;
                for part in parts {
                    if *part != &other[pos..pos + part.len()] {
                        return false;
                    }
                    pos += part.len();
                }
                return true;
            }
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        self.write_to(&mut buffer)
            .expect("writing to Vec should never fail");
        buffer
    }

    pub fn write_encoded<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        match &self.0 {
            _Field::Clean(s) => writer.write_all(s),
            _Field::Escaped(parts) => {
                for part in parts {
                    writer.write_all(&[b'\"'])?;
                    writer.write_all(part)?;
                }
                writer.write_all(&[b'\"'])?;
                Ok(())
            }
        }
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

#[derive(Debug)]
enum _Field<'a> {
    Clean(&'a [u8]),
    Escaped(EscapedIter<'a>),
}

type EscapedIter<'a> = Vec<&'a [u8]>;

#[cfg(test)]
mod tests {
    use super::*;

    mod read_record {
        use super::*;

        fn bbr(bytes: &[u8]) -> std::io::BufReader<std::io::Cursor<&[u8]>> {
            std::io::BufReader::new(std::io::Cursor::new(bytes))
        }

        #[test]
        fn nothing_from_empty_reader() {
            let mut buf = vec![];
            let n = read_record(&mut bbr(b""), &mut buf).unwrap();

            assert_eq!(0, n);
            assert_eq!(b"", buf.as_slice())
        }

        #[test]
        fn two_records_with_lf() {
            let mut buf = vec![];
            let mut reader = bbr(b"a,b,c\n1,2,3");
            let n = read_record(&mut reader, &mut buf).unwrap();

            assert_eq!(6, n);
            assert_eq!(b"a,b,c", buf.as_slice());

            buf.clear();
            let n = read_record(&mut reader, &mut buf).unwrap();

            assert_eq!(5, n);
            assert_eq!(b"1,2,3", buf.as_slice());
        }

        #[test]
        fn single_record_with_lf_at_end() {
            let mut buf = vec![];
            let mut reader = bbr(b"however,I,cannot,say\n");
            let n = read_record(&mut reader, &mut buf).unwrap();

            assert_eq!(21, n);
            assert_eq!(b"however,I,cannot,say", buf.as_slice());
        }

        #[test]
        fn two_records_with_crlf() {
            let mut buf = vec![];
            let mut reader = bbr(b"a,b,c\r\n1,2,3");
            let n = read_record(&mut reader, &mut buf).unwrap();

            assert_eq!(7, n);
            assert_eq!(b"a,b,c", buf.as_slice());

            buf.clear();
            let n = read_record(&mut reader, &mut buf).unwrap();

            assert_eq!(5, n);
            assert_eq!(b"1,2,3", buf.as_slice());
        }

        #[test]
        fn single_record_with_crlf_at_end() {
            let mut buf = vec![];
            let mut reader = bbr(b"however,I,cannot,say\r\n");
            let n = read_record(&mut reader, &mut buf).unwrap();

            assert_eq!(22, n);
            assert_eq!(b"however,I,cannot,say", buf.as_slice());
        }

        #[test]
        fn even_numbers_of_quotes() {
            let mut buf = vec![];
            let mut reader = bbr(b"\"123\"\r\n1\"2\"3\n");
            let n = read_record(&mut reader, &mut buf).unwrap();

            assert_eq!(7, n);
            assert_eq!(b"\"123\"", buf.as_slice());

            buf.clear();
            let n = read_record(&mut reader, &mut buf).unwrap();
            assert_eq!(6, n);
            assert_eq!(b"1\"2\"3", buf.as_slice());
        }

        #[test]
        fn odd_number_of_quotes_reads_to_eof() {
            let mut buf = vec![];
            let mut reader = bbr(b"\"12\"3\"\r\n1\"2\"3\n");
            let n = read_record(&mut reader, &mut buf).unwrap();

            assert_eq!(14, n);
            assert_eq!(b"\"12\"3\"\r\n1\"2\"3\n", buf.as_slice());

            buf.clear();
            let n = read_record(&mut reader, &mut buf).unwrap();
            assert_eq!(0, n);
            assert_eq!(b"", buf.as_slice());
        }

        #[test]
        fn appends_to_buffer() {
            let mut buf = vec![];
            let mut reader = bbr(b"a\nb");
            let n = read_record(&mut reader, &mut buf).unwrap();
            assert_eq!(2, n);
            let n = read_record(&mut reader, &mut buf).unwrap();
            assert_eq!(1, n);
            assert_eq!(b"ab", buf.as_slice());
        }
    }

    mod record_iter {
        use super::*;

        #[test]
        fn empty_string_to_empty_field() {
            let record = Record::new(b"");

            let mut i = record.iter();
            let first = i.next().unwrap().unwrap();
            assert!(first.contents_eq(b""));
            assert!(i.next().is_none());
        }

        #[test]
        fn single_non_empty_field() {
            let record = Record::new(b"hfo d ");

            let mut i = record.iter();
            let first = i.next().unwrap().unwrap();
            assert!(first.contents_eq(b"hfo d "));
            assert!(i.next().is_none());
        }

        #[test]
        fn three_fields() {
            let record = Record::new(b"hfo, d, ");

            let mut i = record.iter();
            assert!(i.next().unwrap().unwrap().contents_eq(b"hfo"));
            assert!(i.next().unwrap().unwrap().contents_eq(b" d"));
            assert!(i.next().unwrap().unwrap().contents_eq(b" "));
            assert!(i.next().is_none());
        }

        #[test]
        fn empty_quoted_field() {
            let record = Record::new(b"\"\"");

            let mut i = record.iter();
            assert!(i.next().unwrap().unwrap().contents_eq(b""));
            assert!(i.next().is_none());
        }

        #[test]
        fn quoted_field_with_comma_therein() {
            let record = Record::new(b"\",\"");

            let mut i = record.iter();
            assert!(i.next().unwrap().unwrap().contents_eq(b","));
            assert!(i.next().is_none());
        }
        #[test]
        fn quoted_field_with_escaped_quote() {
            let record = Record::new(b"\"\"\"\"");

            let mut i = record.iter();
            assert!(i.next().unwrap().unwrap().contents_eq(b"\""));
            assert!(i.next().is_none());
        }

        #[test]
        fn mixed_fields() {
            let record = Record::new(b"hey,\"you, \"\"guys\"\"\",be");

            let mut i = record.iter();
            assert!(i.next().unwrap().unwrap().contents_eq(b"hey"));
            assert!(i.next().unwrap().unwrap().contents_eq(b"you, \"guys\""));
            assert!(i.next().unwrap().unwrap().contents_eq(b"be"));
            assert!(i.next().is_none());
        }
    }

    mod record_read {
        use super::*;

        #[test]
        fn no_expected_field_count() {
            let mut buf = vec![];
            Record::new(b"hey,\"you, \"\"guys\"\"\",be")
                .read(&mut buf, None)
                .unwrap();

            let mut i = buf.iter();
            assert!(i.next().unwrap().contents_eq(b"hey"));
            assert!(i.next().unwrap().contents_eq(b"you, \"guys\""));
            assert!(i.next().unwrap().contents_eq(b"be"));
            assert!(i.next().is_none());
        }

        #[test]
        fn expected_field_count_success() {
            let mut buf = vec![];
            Record::new(b"hey,\"you, \"\"guys\"\"\",be")
                .read(&mut buf, Some(3))
                .unwrap();

            let mut i = buf.iter();
            assert!(i.next().unwrap().contents_eq(b"hey"));
            assert!(i.next().unwrap().contents_eq(b"you, \"guys\""));
            assert!(i.next().unwrap().contents_eq(b"be"));
            assert!(i.next().is_none());
        }

        #[test]
        fn expected_field_count_failure_too_many() {
            let mut buf = vec![];
            assert!(
                Record::new(b"hey,\"you, \"\"guys\"\"\",be")
                    .read(&mut buf, Some(2))
                    .is_err()
            );
        }

        #[test]
        fn expected_field_count_failure_too_few() {
            let mut buf = vec![];
            assert!(
                Record::new(b"hey,\"you, \"\"guys\"\"\",be")
                    .read(&mut buf, Some(4))
                    .is_err()
            );
        }
    }
}
