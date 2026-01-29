
use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub fn load_csv_data(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: DataRecord = result?;
        records.push(record);
    }

    Ok(records)
}

pub fn filter_by_category(records: &[DataRecord], category: &str) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|record| record.category == category)
        .cloned()
        .collect()
}

pub fn calculate_average_value(records: &[DataRecord]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|record| record.value).sum();
    Some(sum / records.len() as f64)
}

pub fn save_filtered_data(
    records: &[DataRecord],
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut writer = Writer::from_writer(file);

    for record in records {
        writer.serialize(record)?;
    }

    writer.flush()?;
    Ok(())
}

pub fn process_data_pipeline(
    input_path: &str,
    category_filter: &str,
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let all_records = load_csv_data(input_path)?;
    let filtered_records = filter_by_category(&all_records, category_filter);

    if let Some(avg) = calculate_average_value(&filtered_records) {
        println!("Average value for category '{}': {:.2}", category_filter, avg);
    }

    save_filtered_data(&filtered_records, output_path)?;
    println!("Filtered data saved to: {}", output_path);

    Ok(())
}