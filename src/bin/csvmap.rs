use std::io::Write;

use csvtools::csv::{read, write::encode_and_write_quoted};

const PARAM_USAGE: &str = "source_column[:destination_column] command [arg] ...";
fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let program_name = args.get(0).expect("the first argument to exist");
    let usage = format!("{} {}", program_name, PARAM_USAGE);

    if args.len() < 3 {
        err(&format!("Usage: {}", usage))
    }

    let (source_colum, destination_column) = get_source_and_destination(&args[1]);

    let mut csv_file = read::File::new(std::io::stdin());

    let mut binding = std::process::Command::new(&args[2]);
    let command = binding
        .args(&args[3..])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped());

    let Ok(Some(header)) = csv_file.read() else {
        err("could not header from standard input.");
    };

    let columns = header.iter().map(|f| f.unwrap()).collect::<Vec<_>>();
    let source_idx = columns
        .iter()
        .position(|f| f.contents_eq(source_colum.as_bytes()))
        .unwrap_or_else(|| err(&format!("`{}` not found in the header.", source_colum)));
    let map_in_place = source_colum == destination_column;
    assert!(map_in_place);

    let num_in_columns = columns.len();

    let mut stdout = std::io::stdout();

    for (i, field) in columns.iter().enumerate() {
        if i != 0 {
            stdout.write_all(&[b',']).expect("write");
        }
        field.write_to(&mut stdout).expect("write");
    }

    println!();

    while let Ok(Some(row)) = csv_file.read() {
        let mut i = 0;
        for field in row.iter() {
            if i == num_in_columns {
                i += 1;
                break;
            }
            if i != 0 {
                stdout.write_all(&[b',']).expect("write");
            }
            let Ok(field) = field else {
                err(&format!("Error reading field {}.", i))
            };

            if i == source_idx {
                let mut proc = command.spawn().expect("proc to run");
                let mut proc_stdin = proc.stdin.take().expect("stdin to be available");
                let proc_stdout = proc.stdout.take().expect("stdin to be available");
                field.write_to(&mut proc_stdin).expect("write");
                drop(proc_stdin);
                proc.wait().expect("program closes");
                encode_and_write_quoted(&stdout, proc_stdout).expect("write");
            } else {
                field.write_encoded(&mut stdout).expect("write");
            }

            i += 1;
        }
        println!();

        if i != num_in_columns {
            err(&format!(
                "Expected {} columns but found {}",
                num_in_columns, i
            ));
        }
    }
}

fn err(s: &str) -> ! {
    eprintln!("{}", s);
    std::process::exit(1)
}

fn get_source_and_destination(s: &str) -> (&str, &str) {
    let mut sp = s.split(":");
    let source = sp.next().unwrap();
    let destination = sp.next().unwrap_or(source);
    return (source, destination);
}
