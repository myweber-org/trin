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

fn filter_records(input_path: &str, output_path: &str, min_age: u8) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(input_file);

    let output_file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(output_file);

    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.age >= min_age {
            wtr.serialize(&record)?;
        }
    }

    wtr.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    filter_records("input.csv", "output.csv", 25)?;
    println!("Filtered records saved to output.csv");
    Ok(())
}