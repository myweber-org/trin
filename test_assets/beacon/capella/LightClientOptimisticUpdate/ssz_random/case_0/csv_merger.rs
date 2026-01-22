use std::error::Error;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

pub fn merge_csv_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
    write_headers: bool,
) -> Result<(), Box<dyn Error>> {
    let mut writer = BufWriter::new(File::create(output_path)?);
    let mut first_file = true;

    for path in input_paths {
        let mut reader = csv::Reader::from_path(path)?;
        let headers = reader.headers()?.clone();

        if first_file {
            if write_headers {
                writer.write_all(headers.as_bytes())?;
                writer.write_all(b"\n")?;
            }
            first_file = false;
        }

        for result in reader.records() {
            let record = result?;
            writer.write_all(record.as_slice())?;
            writer.write_all(b"\n")?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let inputs = vec![
        Path::new("data1.csv"),
        Path::new("data2.csv"),
        Path::new("data3.csv"),
    ];
    let output = Path::new("merged_output.csv");

    merge_csv_files(&inputs, output, true)?;
    println!("CSV files merged successfully into {:?}", output);
    Ok(())
}