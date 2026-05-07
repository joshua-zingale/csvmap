use std::io::BufRead;

pub struct File<R: std::io::Read> {
    reader: std::io::BufReader<R>,
    buffer: String,
    first: bool,
}

impl<T: std::io::Read> File<T> {
    pub fn new(reader: T) -> Self {
        File {
            reader: std::io::BufReader::new(reader),
            buffer: String::new(),
            first: true,
        }
    }

    pub fn read<'a>(&'a mut self) -> Option<Row<'a>> {
        self.buffer.clear();
        let first = self.first;
        self.first = false;

        match self.reader.read_line(&mut self.buffer) {
            Ok(0) if first => Some(Row(self.buffer.as_str())),
            Ok(0) => None,
            Ok(_) => {
                if self.buffer.as_bytes()[self.buffer.len() - 1] == b'\n' {
                    self.buffer.pop();
                    if let Some(&last) = self.buffer.as_bytes().last()
                        && last == b'\r'
                    {
                        self.buffer.pop();
                    }
                }
                Some(Row(self.buffer.as_str()))
            }
            Err(_) => None,
        }
    }
}

pub struct Row<'a>(&'a str);

impl<'a> Row<'a> {
    pub fn iter(&self) -> RowIter<'a> {
        RowIter {
            rest: self.0,
            is_next: true,
        }
    }
}

pub struct RowIter<'a> {
    rest: &'a str,
    is_next: bool,
}

impl<'a> Iterator for RowIter<'a> {
    type Item = Field<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_next {
            return None;
        }

        if !self.rest.starts_with('"') {
            if let Some(comma_pos) = self.rest.bytes().position(|c| c == b',') {
                let field = _Field::Clean(&self.rest[..comma_pos]);
                self.rest = &self.rest[comma_pos + 1..];
                Some(Field(field))
            } else {
                self.is_next = false;
                Some(Field(_Field::Clean(self.rest)))
            }
        } else {
            let mut parts = Vec::new();

            let mut rest = self.rest;
            loop {
                if let Some(double_quote_pos) = rest.bytes().position(|c| c == b'"') {
                    if let Some(&c) = rest.as_bytes().get(1)
                        && c == b'"'
                    {
                        parts.push(&rest[..double_quote_pos + 1]);
                        rest = &rest[double_quote_pos + 2..]
                    } else if let Some(comma_pos) = rest.bytes().position(|c| c == b',') {
                        parts.push(&rest[..comma_pos]);
                        rest = &rest[comma_pos + 1..];
                        break;
                    } else {
                        self.is_next = false;
                        parts.push(rest);
                        break;
                    }
                } else {
                    self.is_next = false;
                    parts.push(rest);
                    break;
                }
            }

            self.rest = rest;
            Some(Field(_Field::Escaped(parts)))
        }
    }
}

pub struct Field<'a>(_Field<'a>);

impl<'a> Field<'a> {
    pub fn get_string(&self) -> String {
        match &self.0 {
            _Field::Clean(s) => s.to_string(),
            _Field::Escaped(parts) => parts.join(""),
        }
    }
}

enum _Field<'a> {
    Clean(&'a str),
    Escaped(EscapedIter<'a>),
}

type EscapedIter<'a> = Vec<&'a str>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_three_rows() {
        let mut f = File::new("a,b\n1,2\n3,4".as_bytes());

        assert!(f.read().is_some());
        assert!(f.read().is_some());
        assert!(f.read().is_some());
        assert!(f.read().is_none());
    }

    #[test]
    fn reads_one_row_for_empty_string() {
        let mut f = File::new("".as_bytes());

        assert!(f.read().is_some());
        assert!(f.read().is_none());
    }

    #[test]
    fn reads_one_row_for_single_newline() {
        let mut f = File::new("\n".as_bytes());

        assert!(f.read().is_some());
        assert!(f.read().is_none());
    }

    #[test]
    fn reads_one_row() {
        let one_rows = vec!["a,b,c,", "a,b,", "a,b", "aasdf"]
            .into_iter()
            .map(|s| s.as_bytes());

        for row in one_rows {
            let mut f = File::new(row);
            assert!(f.read().is_some());
            assert!(f.read().is_none());
        }
    }

    #[test]
    fn reads_values() {
        todo!();
    }
}
