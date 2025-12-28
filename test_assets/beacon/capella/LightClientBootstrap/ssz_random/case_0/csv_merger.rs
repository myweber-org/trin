use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: {} <output_file> <input_file1> [input_file2 ...]", args[0]);
        std::process::exit(1);
    }

    let output_path = &args[1];
    let input_paths = &args[2..];

    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    let mut headers_written = false;

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
                writer.write_all(header.as_bytes())?;
                writer.write_all(b"\n")?;
                headers_written = true;
            } else if header != get_first_line(input_paths[0])? {
                eprintln!("Warning: Headers differ between files. Using header from first file.");
            }

            for line in lines {
                let line = line?;
                if !line.trim().is_empty() {
                    writer.write_all(line.as_bytes())?;
                    writer.write_all(b"\n")?;
                }
            }
        }
    }

    if headers_written {
        println!("Successfully merged {} files into '{}'", input_paths.len(), output_path);
    } else {
        eprintln!("Error: No valid input files processed.");
    }

    Ok(())
}

fn get_first_line(file_path: &str) -> io::Result<String> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let first_line = reader.lines().next().transpose()?.unwrap_or_default();
    Ok(first_line)
}