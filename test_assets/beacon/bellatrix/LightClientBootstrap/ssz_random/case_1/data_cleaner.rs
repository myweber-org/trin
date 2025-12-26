use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u8,
    active: bool,
}

struct DataCleaner {
    input_path: String,
    output_path: String,
    min_age: u8,
}

impl DataCleaner {
    fn new(input_path: &str, output_path: &str, min_age: u8) -> Self {
        DataCleaner {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            min_age,
        }
    }

    fn clean_data(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(input_file);

        let output_file = File::create(&self.output_path)?;
        let mut wtr = WriterBuilder::new()
            .has_headers(true)
            .from_writer(output_file);

        let mut processed_count = 0;

        for result in rdr.deserialize() {
            let record: Record = result?;
            
            if record.age >= self.min_age && record.active {
                wtr.serialize(&record)?;
                processed_count += 1;
            }
        }

        wtr.flush()?;
        Ok(processed_count)
    }

    fn validate_paths(&self) -> Result<(), Box<dyn Error>> {
        if !Path::new(&self.input_path).exists() {
            return Err("Input file does not exist".into());
        }

        let output_dir = Path::new(&self.output_path)
            .parent()
            .ok_or("Invalid output path")?;

        if !output_dir.exists() {
            return Err("Output directory does not exist".into());
        }

        Ok(())
    }
}

fn process_dataset() -> Result<(), Box<dyn Error>> {
    let cleaner = DataCleaner::new("input.csv", "output/cleaned.csv", 18);
    
    cleaner.validate_paths()?;
    
    let processed = cleaner.clean_data()?;
    println!("Processed {} valid records", processed);
    
    Ok(())
}

fn main() {
    if let Err(e) = process_dataset() {
        eprintln!("Error processing data: {}", e);
        std::process::exit(1);
    }
}