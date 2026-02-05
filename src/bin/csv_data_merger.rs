use std::error::Error;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

pub fn merge_csv_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
    write_headers: bool,
) -> Result<(), Box<dyn Error>> {
    let mut output_writer = BufWriter::new(File::create(output_path)?);
    let mut first_file = true;

    for path in input_paths {
        let mut rdr = csv::Reader::from_path(path)?;
        let headers = rdr.headers()?.clone();

        if first_file {
            if write_headers {
                writeln!(output_writer, "{}", headers.as_str())?;
            }
            first_file = false;
        }

        for result in rdr.records() {
            let record = result?;
            writeln!(output_writer, "{}", record.as_str())?;
        }
    }

    output_writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_files = vec![
        "data/file1.csv",
        "data/file2.csv",
        "data/file3.csv",
    ];
    let output_file = "merged_data.csv";

    merge_csv_files(&input_files, output_file, true)?;
    println!("CSV files merged successfully into '{}'", output_file);
    Ok(())
}