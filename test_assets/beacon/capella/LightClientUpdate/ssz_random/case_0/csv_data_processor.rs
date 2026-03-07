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

fn load_records(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}

fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter().filter(|r| r.active).collect()
}

fn calculate_category_totals(records: &[Record]) -> Vec<(String, f64)> {
    let mut totals = std::collections::HashMap::new();

    for record in records {
        let entry = totals.entry(record.category.clone()).or_insert(0.0);
        *entry += record.value;
    }

    totals.into_iter().collect()
}

fn save_processed_data(
    active_records: &[&Record],
    category_totals: &[(String, f64)],
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(output_path)?;

    wtr.write_record(&["ID", "Name", "Category", "Value", "Status"])?;
    for record in active_records {
        wtr.write_record(&[
            record.id.to_string(),
            record.name.clone(),
            record.category.clone(),
            record.value.to_string(),
            "Active".to_string(),
        ])?;
    }

    wtr.write_record(&[])?;
    wtr.write_record(&["Category", "Total Value"])?;
    for (category, total) in category_totals {
        wtr.write_record(&[category.clone(), total.to_string()])?;
    }

    wtr.flush()?;
    Ok(())
}

fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let records = load_records(input_path)?;
    let active_records = filter_active_records(&records);
    let category_totals = calculate_category_totals(&records);

    save_processed_data(&active_records, &category_totals, output_path)?;

    println!("Processed {} records", records.len());
    println!("Found {} active records", active_records.len());
    println!("Calculated totals for {} categories", category_totals.len());

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/processed_output.csv";

    match process_csv_data(input_file, output_file) {
        Ok(_) => println!("Data processing completed successfully"),
        Err(e) => eprintln!("Error processing data: {}", e),
    }

    Ok(())
}