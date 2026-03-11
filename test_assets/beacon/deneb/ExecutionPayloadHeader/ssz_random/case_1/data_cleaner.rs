use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    for result in reader.deserialize() {
        let mut record: Record = result?;
        
        record.name = record.name.trim().to_string();
        record.category = record.category.to_lowercase();
        
        if record.value < 0.0 {
            record.value = 0.0;
        }
        
        writer.serialize(&record)?;
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    clean_csv("input.csv", "cleaned_output.csv")?;
    println!("Data cleaning completed successfully");
    Ok(())
}