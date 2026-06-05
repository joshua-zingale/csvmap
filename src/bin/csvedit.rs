use std::io::Write;

use csvtools::common::arglex;
fn main() {
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
                    reader = Either::B(std::fs::File::open(filename).unwrap_or_else(|_| {
                        eprintln!("could not open `{filename}`.");
                        std::process::exit(1)
                    }))
                }
                Either::B(_) => {
                    eprintln!("invalid usage: only one positional argument is accepted.");
                    std::process::exit(1);
                }
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

    let mut csv_file = csvtools::csv::read::File::new(reader);

    let num_fields;
    let out_indices: Vec<_> = if has_header {
        let header = match csv_file.read() {
            Ok(Some(header)) => header,
            Ok(None) => unreachable!("empty files should always return a single empty column name"),
            Err(_) => {
                eprintln!("could not read from csv file.");
                std::process::exit(1);
            }
        };

        let header: Vec<_> = header
            .iter()
            .map(|f| {
                let Ok(f) = f else {
                    eprintln!("Bad field");
                    std::process::exit(1);
                };
                f
            })
            .collect();

        num_fields = header.len();

        let indices: Vec<_> = columns
            .iter()
            .map(|name| {
                header
                    .iter()
                    .enumerate()
                    .find(|(_, f)| f.contents_eq(name.as_bytes()))
                    .map(|(i, _)| i)
                    .unwrap_or_else(|| {
                        eprintln!("Could not find field `{}`", name);
                        std::process::exit(1);
                    })
            })
            .collect();

        for (i, &idx) in indices.iter().enumerate() {
            if i != 0 {
                let _ = writer.write_all(&[b',']);
            }
            header[idx].write_encoded(&mut writer).unwrap();
        }
        indices
    } else {
        todo!()
    };

    writer.write_all(&[b'\n']).unwrap();

    loop {
        match csv_file.read() {
            Ok(None) => break,
            Err(_) => {
                eprintln!("could not read file.");
                std::process::exit(1);
            }
            Ok(Some(record)) => {
                let fields: Vec<_> = record
                    .iter()
                    .map(|f| {
                        let Ok(f) = f else {
                            eprint!("Invalid field.");
                            std::process::exit(1);
                        };
                        f
                    })
                    .collect();
                if fields.len() != num_fields {
                    eprintln!(
                        "`{}` fields found but `{}` required.",
                        fields.len(),
                        num_fields,
                    );
                    std::process::exit(1);
                }

                for (i, &idx) in out_indices.iter().enumerate() {
                    if i != 0 {
                        let _ = writer.write_all(&[b',']);
                    }
                    fields[idx].write_encoded(&mut writer).unwrap();
                }

                writer.write_all(&[b'\n']).unwrap();
            }
        }
    }
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
