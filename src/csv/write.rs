use std::io::{BufReader, BufWriter, Read, Write};

/// Encodes data from a reader as a quoted CSV field and writes them to a writer.
/// Does not flush writer.
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
            written += 2
        } else {
            writer.write_all(&[c])?;
            written += 1
        }
    }

    writer.write_all(b"\"")?;
    written += 1;
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

    #[test]
    fn num_written_no_quotes() {
        let mut s = Vec::new();
        let n = encode_and_write_quoted(&mut s, std::io::Cursor::new(b"123456")).unwrap();
        assert_eq!(n, 8);
    }

    #[test]
    fn num_written_2_for_empty_string() {
        let mut s = Vec::new();
        let n = encode_and_write_quoted(&mut s, std::io::Cursor::new(b"")).unwrap();
        assert_eq!(n, 2);
    }

    #[test]
    fn num_written_4_with_single_inner_quote() {
        let mut s = Vec::new();
        let n = encode_and_write_quoted(&mut s, std::io::Cursor::new(b"\"")).unwrap();
        assert_eq!(n, 4);
    }

    #[test]
    fn num_written_with_quotes() {
        let mut s = Vec::new();
        let n = encode_and_write_quoted(
            &mut s,
            std::io::Cursor::new(b"He said, \"I love thee above all maids.\""),
        )
        .unwrap();
        assert_eq!(n, 43);
    }
}
