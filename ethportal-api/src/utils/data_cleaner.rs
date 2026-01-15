use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    let mut cleaned_count = 0;
    let mut error_count = 0;

    for result in rdr.deserialize() {
        match result {
            Ok(mut record) => {
                let rec: Record = record;
                let cleaned_record = Record {
                    name: rec.name.trim().to_string(),
                    value: rec.value.max(0.0),
                    ..rec
                };
                wtr.serialize(&cleaned_record)?;
                cleaned_count += 1;
            }
            Err(e) => {
                eprintln!("Skipping invalid record: {}", e);
                error_count += 1;
            }
        }
    }

    wtr.flush()?;
    println!("Cleaned {} records, skipped {} errors", cleaned_count, error_count);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    clean_csv_data("input.csv", "output.csv")
}