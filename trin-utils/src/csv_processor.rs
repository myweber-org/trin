
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn process_csv(input_path: &Path, output_path: &Path, min_value: f64) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let reader = BufReader::new(file);
    let mut csv_reader = csv::Reader::from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = csv::Writer::from_writer(writer);

    for result in csv_reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= min_value && record.active {
            let transformed_record = Record {
                name: record.name.to_uppercase(),
                category: record.category.replace("old", "new"),
                value: (record.value * 100.0).round() / 100.0,
                ..record
            };
            csv_writer.serialize(transformed_record)?;
        }
    }

    csv_writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = Path::new("data/input.csv");
    let output_path = Path::new("data/filtered_output.csv");
    let threshold = 50.0;

    process_csv(input_path, output_path, threshold)?;
    
    println!("CSV processing completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv() {
        let csv_data = "id,name,category,value,active\n\
                        1,test item,old_category,75.5,true\n\
                        2,another item,some_category,30.0,true\n\
                        3,inactive item,category,80.0,false";

        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "{}", csv_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let result = process_csv(
            input_file.path(),
            output_file.path(),
            50.0
        );
        
        assert!(result.is_ok());
        
        let output_content = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(output_content.contains("TEST ITEM"));
        assert!(!output_content.contains("another item"));
        assert!(!output_content.contains("inactive item"));
        assert!(output_content.contains("new_category"));
    }
}