use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn process_csv(input_path: &str, output_path: &str, min_value: f64) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = csv::Reader::from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = csv::Writer::from_writer(writer);

    for result in csv_reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= min_value && record.active {
            let transformed_record = Record {
                name: record.name.to_uppercase(),
                category: record.category.trim().to_string(),
                ..record
            };
            csv_writer.serialize(transformed_record)?;
        }
    }

    csv_writer.flush()?;
    Ok(())
}

fn main() {
    let input_file = "data/input.csv";
    let output_file = "data/filtered_output.csv";
    let threshold = 100.0;

    match process_csv(input_file, output_file, threshold) {
        Ok(()) => println!("Processing completed successfully"),
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }
}