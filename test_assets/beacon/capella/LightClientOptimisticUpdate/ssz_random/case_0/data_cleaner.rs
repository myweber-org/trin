use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(reader);

    let output_file = File::create(Path::new(output_path))?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(writer);

    for result in csv_reader.deserialize() {
        let record: Record = result?;
        
        let cleaned_record = Record {
            id: record.id,
            name: record.name.trim().to_string(),
            value: if record.value.is_nan() || record.value.is_infinite() {
                0.0
            } else {
                record.value
            },
            category: if record.category.is_empty() {
                "unknown".to_string()
            } else {
                record.category.to_lowercase()
            },
        };

        csv_writer.serialize(cleaned_record)?;
    }

    csv_writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "raw_data.csv";
    let output_file = "cleaned_data.csv";

    match clean_csv_data(input_file, output_file) {
        Ok(_) => println!("Data cleaning completed successfully"),
        Err(e) => eprintln!("Error during data cleaning: {}", e),
    }

    Ok(())
}