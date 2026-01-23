use csv::ReaderBuilder;
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
    let file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let mut valid_records = Vec::new();
    let mut invalid_count = 0;

    for result in rdr.deserialize() {
        match result {
            Ok(record) => {
                let rec: Record = record;
                if validate_record(&rec) {
                    valid_records.push(rec);
                } else {
                    invalid_count += 1;
                }
            }
            Err(e) => {
                eprintln!("Skipping malformed record: {}", e);
                invalid_count += 1;
            }
        }
    }

    println!("Processing complete:");
    println!("  Valid records: {}", valid_records.len());
    println!("  Invalid records: {}", invalid_count);

    if !valid_records.is_empty() {
        let mut wtr = csv::Writer::from_path(output_path)?;
        for record in valid_records {
            wtr.serialize(record)?;
        }
        wtr.flush()?;
        println!("Cleaned data written to: {}", output_path);
    }

    Ok(())
}

fn validate_record(record: &Record) -> bool {
    !record.name.trim().is_empty() &&
    record.value >= 0.0 &&
    !record.category.is_empty() &&
    record.id > 0
}