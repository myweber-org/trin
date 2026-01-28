
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: {} <output_file> <input_file1> [input_file2 ...]", args[0]);
        std::process::exit(1);
    }

    let output_path = &args[1];
    let input_paths = &args[2..];

    let mut output_file = File::create(output_path)?;
    let mut header_written = false;

    for (i, input_path) in input_paths.iter().enumerate() {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File '{}' not found, skipping.", input_path);
            continue;
        }

        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(first_line) = lines.next() {
            let header = first_line?;
            
            if i == 0 {
                writeln!(output_file, "{}", header)?;
                header_written = true;
            } else if !header_written {
                writeln!(output_file, "{}", header)?;
                header_written = true;
            }

            for line in lines {
                writeln!(output_file, "{}", line?)?;
            }
        }
    }

    println!("Successfully merged {} files into '{}'", input_paths.len(), output_path);
    Ok(())
}