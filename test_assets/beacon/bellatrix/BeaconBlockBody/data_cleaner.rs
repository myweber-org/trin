use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    let mut valid_records = Vec::new();
    let mut invalid_count = 0;

    for result in reader.deserialize() {
        let record: Record = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Invalid record skipped: {}", e);
                invalid_count += 1;
                continue;
            }
        };

        if record.value >= 0.0 && !record.name.is_empty() {
            valid_records.push(record);
        } else {
            invalid_count += 1;
        }
    }

    let mut writer = csv::Writer::from_path(output_path)?;
    for record in valid_records {
        writer.serialize(record)?;
    }
    writer.flush()?;

    println!("Processing complete:");
    println!("  Valid records: {}", valid_records.len());
    println!("  Invalid records: {}", invalid_count);

    Ok(())
}