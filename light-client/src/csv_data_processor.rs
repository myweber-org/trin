use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn filter_and_aggregate(input_path: &str, output_path: &str, category_filter: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_writer(File::create(output_path)?);

    let mut total_value = 0.0;
    let mut record_count = 0;

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.category == category_filter && record.active {
            writer.serialize(&record)?;
            total_value += record.value;
            record_count += 1;
        }
    }

    if record_count > 0 {
        let average = total_value / record_count as f64;
        println!("Processed {} records in category '{}'", record_count, category_filter);
        println!("Total value: {:.2}, Average: {:.2}", total_value, average);
    } else {
        println!("No records found for category '{}'", category_filter);
    }

    writer.flush()?;
    Ok(())
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && record.value >= 0.0
}

fn process_data() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/filtered_output.csv";
    let target_category = "electronics";

    match filter_and_aggregate(input_file, output_file, target_category) {
        Ok(_) => println!("Data processing completed successfully"),
        Err(e) => eprintln!("Error processing data: {}", e),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_record() {
        let valid_record = Record {
            id: 1,
            name: String::from("Test"),
            category: String::from("test"),
            value: 10.0,
            active: true,
        };
        
        let invalid_record = Record {
            id: 2,
            name: String::new(),
            category: String::from("test"),
            value: -5.0,
            active: true,
        };

        assert!(validate_record(&valid_record));
        assert!(!validate_record(&invalid_record));
    }
}