use csv::{Reader, Writer};
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

fn filter_and_transform(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.age >= 18 && record.active {
            let transformed = Record {
                id: record.id,
                name: record.name.to_uppercase(),
                age: record.age,
                active: record.active,
            };
            writer.serialize(transformed)?;
        }
    }
    
    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    filter_and_transform("input.csv", "output.csv")?;
    println!("Processing completed successfully");
    Ok(())
}