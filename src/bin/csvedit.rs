use std::io::{BufReader, Write};

use csvtools::common::arglex;
use csvtools::common::error::ErrorContextAdd;
use csvtools::csv;

fn main() -> csvtools::common::cmd::MainResult {
    let mut has_header = true;

    let mut writer = std::io::stdout();
    let mut reader = Either::A(std::io::stdin());
    let mut columns = Vec::new();
    let args: Vec<_> = std::env::args().collect();
    let arg_slices = args[1..].iter().map(|s| s.as_str());
    let mut arg_lexer = arglex::ArgLexer::new(arg_slices);
    while let Some(arg) = arg_lexer.next() {
        use arglex::Arg::*;
        match arg {
            Positional(filename) => match reader {
                Either::A(_) => {
                    reader = Either::B(
                        std::fs::File::open(filename)
                            .map_err(|e| e.add_context(format!("openning `{}`", filename)))?,
                    )
                }
                Either::B(_) => Err("only one positional argument is accepted")?,
            },
            Short('c') => {
                let arg = arg_lexer.consume_arg().unwrap_or_else(|| {
                    eprintln!("`-c` expects an argument");
                    std::process::exit(1);
                });
                columns.push(arg);
            }
            Long("header") => has_header = true,
            Short('n') | Long("no-header") => has_header = false,
            Short(c) => {
                eprintln!("invalid option `-{c}`");
                std::process::exit(1);
            }
            Long(s) => {
                eprintln!("invalid option `--{s}`");
                std::process::exit(1);
            }
            LongArg(s, a) => {
                eprintln!("invalid option with argument `--{s}={a}`");
                std::process::exit(1);
            }
        }
    }

    let mut reader = BufReader::new(reader);

    let mut header = Vec::with_capacity(128);
    let mut header_fields = Vec::with_capacity(16);
    let num_fields;
    let out_indices: Vec<_> = if has_header {
        let _ = csv::read::read_record(&mut reader, &mut header)?;

        csv::read::parse_fields(&mut header, &mut header_fields, None)
            .map_err(|e| e.record_num(0))?;

        num_fields = header_fields.len();

        let indices: Vec<_> = columns
            .iter()
            .map(|name| {
                header_fields
                    .iter()
                    .enumerate()
                    .find(|(_, f)| f.contents_eq(name.as_bytes()))
                    .map(|(i, _)| i)
                    .unwrap_or_else(|| {
                        eprintln!("Error: no column named `{}`", name);
                        std::process::exit(1);
                    })
            })
            .collect();

        for (i, &idx) in indices.iter().enumerate() {
            if i != 0 {
                writer.write_all(&[b','])?;
            }
            header_fields[idx].write_encoded(&mut writer)?;
        }
        indices
    } else {
        todo!()
    };

    writeln!(writer)?;

    let mut reader = csv::read::RecordReader::new(reader, Some(num_fields));
    reader.next_record_num += 1;
    while let Some(fields) = reader.read()? {
        for (i, &idx) in out_indices.iter().enumerate() {
            if i != 0 {
                let _ = writer.write_all(&[b',']);
            }
            fields[idx].write_encoded(&mut writer)?;
        }
        writeln!(writer)?;
    }

    Ok(())
}

enum Either<A, B> {
    A(A),
    B(B),
}

impl<A, B> std::io::Read for Either<A, B>
where
    A: std::io::Read,
    B: std::io::Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Either::A(a) => a.read(buf),
            Either::B(b) => b.read(buf),
        }
    }
}
