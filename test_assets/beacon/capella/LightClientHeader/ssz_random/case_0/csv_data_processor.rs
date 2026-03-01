
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

#[derive(Debug)]
struct AggregatedData {
    category: String,
    total_value: f64,
    average_value: f64,
    record_count: usize,
    active_count: usize,
}

fn load_csv_data(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
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

fn aggregate_by_category(records: &[Record]) -> Vec<AggregatedData> {
    use std::collections::HashMap;

    let mut category_map: HashMap<String, (f64, usize, usize)> = HashMap::new();

    for record in records {
        let entry = category_map.entry(record.category.clone()).or_insert((0.0, 0, 0));
        entry.0 += record.value;
        entry.1 += 1;
        if record.active {
            entry.2 += 1;
        }
    }

    category_map
        .into_iter()
        .map(|(category, (total, count, active_count))| AggregatedData {
            category,
            total_value: total,
            average_value: total / count as f64,
            record_count: count,
            active_count,
        })
        .collect()
}

fn save_aggregated_data(
    aggregated: &[AggregatedData],
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut writer = Writer::from_writer(file);

    for data in aggregated {
        writer.serialize(data)?;
    }

    writer.flush()?;
    Ok(())
}

fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let records = load_csv_data(input_path)?;
    let active_records = filter_active_records(&records);
    let aggregated_data = aggregate_by_category(&records);

    println!("Total records loaded: {}", records.len());
    println!("Active records: {}", active_records.len());
    println!("Categories aggregated: {}", aggregated_data.len());

    for data in &aggregated_data {
        println!(
            "Category: {}, Total: {:.2}, Avg: {:.2}, Records: {}, Active: {}",
            data.category,
            data.total_value,
            data.average_value,
            data.record_count,
            data.active_count
        );
    }

    save_aggregated_data(&aggregated_data, output_path)?;
    println!("Aggregated data saved to: {}", output_path);

    Ok(())
}

pub fn run_processor() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output_aggregated.csv";

    match process_csv_data(input_file, output_file) {
        Ok(_) => println!("CSV processing completed successfully"),
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }

    Ok(())
}