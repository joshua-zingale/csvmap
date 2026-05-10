use std::io::{BufReader, BufWriter, Read, Write};

/// Encodes data from a reader as a quoted CSV field and writes them to a writer.
pub fn encode_and_write_quoted<R: Read, W: Write>(writer: W, reader: R) -> std::io::Result<usize> {
    let reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);
    let mut written = 0;

    writer.write_all(b"\"")?;
    written += 1;

    for c in reader.bytes() {
        let c = c?;

        if c == b'"' {
            writer.write_all(&[c, c])?;
        } else {
            writer.write_all(&[c])?;
        }
    }

    writer.write_all(b"\"")?;
    writer.flush()?;
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encodes_with_none_escaped() {
        let mut s = Vec::new();
        encode_and_write_quoted(&mut s, std::io::Cursor::new(b"hello you!")).unwrap();
        assert_eq!(b"\"hello you!\"", s.as_slice())
    }

    #[test]
    fn encodes_with_escaped() {
        let mut s = Vec::new();
        encode_and_write_quoted(&mut s, std::io::Cursor::new(b"\"hello\" you!")).unwrap();
        assert_eq!(b"\"\"\"hello\"\" you!\"", s.as_slice())
    }
}
