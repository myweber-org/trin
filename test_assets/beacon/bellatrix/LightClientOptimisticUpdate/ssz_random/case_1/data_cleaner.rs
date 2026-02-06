use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u8,
    active: bool,
}

fn clean_record(record: &mut Record) {
    record.name = record.name.trim().to_string();
    if record.age > 120 {
        record.age = 120;
    }
}

fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(input_file);

    let output_file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(output_file);

    for result in rdr.deserialize() {
        let mut record: Record = result?;
        clean_record(&mut record);
        wtr.serialize(&record)?;
    }

    wtr.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = "data/raw.csv";
    let output = "data/cleaned.csv";

    match process_csv(input, output) {
        Ok(_) => println!("Data cleaning completed successfully."),
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }

    Ok(())
}