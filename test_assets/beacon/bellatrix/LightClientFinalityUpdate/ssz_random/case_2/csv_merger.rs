use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn merge_csv_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn Error>> {
    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    let mut headers_written = false;

    for (index, input_path) in input_paths.iter().enumerate() {
        let file = File::open(input_path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let headers = rdr.headers()?.clone();

        if index == 0 {
            writer.write_all(headers.as_bytes())?;
            writer.write_all(b"\n")?;
            headers_written = true;
        } else if headers != rdr.headers()? {
            eprintln!("Warning: Headers in {} differ from first file.", input_path);
            if !headers_written {
                writer.write_all(headers.as_bytes())?;
                writer.write_all(b"\n")?;
                headers_written = true;
            }
        }

        for result in rdr.records() {
            let record = result?;
            writer.write_all(record.as_slice().as_bytes())?;
            writer.write_all(b"\n")?;
        }
    }

    writer.flush()?;
    println!("Successfully merged {} files into {}", input_paths.len(), output_path);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let inputs = vec!["data1.csv", "data2.csv", "data3.csv"];
    let output = "merged_data.csv";

    if !inputs.iter().all(|p| Path::new(p).exists()) {
        eprintln!("Error: One or more input files do not exist.");
        std::process::exit(1);
    }

    merge_csv_files(&inputs, output)
}