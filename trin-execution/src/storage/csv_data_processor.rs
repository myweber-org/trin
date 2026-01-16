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

fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}

fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter().filter(|r| r.active).collect()
}

fn calculate_category_averages(records: &[Record]) -> Vec<(String, f64)> {
    use std::collections::HashMap;

    let mut category_totals: HashMap<String, (f64, usize)> = HashMap::new();

    for record in records {
        let entry = category_totals
            .entry(record.category.clone())
            .or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }

    category_totals
        .into_iter()
        .map(|(category, (total, count))| (category, total / count as f64))
        .collect()
}

fn save_processed_data(
    records: &[&Record],
    averages: &[(String, f64)],
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(output_path)?;

    writer.write_record(&["ID", "Name", "Category", "Value", "Status"])?;
    for record in records {
        writer.serialize((
            record.id,
            &record.name,
            &record.category,
            record.value,
            "Active",
        ))?;
    }

    writer.write_record(&[])?;
    writer.write_record(&["Category", "Average Value"])?;
    for (category, avg) in averages {
        writer.serialize((category, avg))?;
    }

    writer.flush()?;
    Ok(())
}

fn process_csv_data(input_file: &str, output_file: &str) -> Result<(), Box<dyn Error>> {
    let records = load_csv(input_file)?;
    let active_records = filter_active_records(&records);
    let category_averages = calculate_category_averages(&records);

    save_processed_data(&active_records, &category_averages, output_file)?;

    println!("Processed {} records", records.len());
    println!("Found {} active records", active_records.len());
    println!("Calculated averages for {} categories", category_averages.len());

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