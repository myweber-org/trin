use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
pub struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, category: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            category,
            value,
            active,
        }
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn category(&self) -> &str {
        &self.category
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}

pub fn load_records_from_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

    let mut records = Vec::new();

    for result in csv_reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}

pub fn filter_records_by_category(records: &[Record], category_filter: &str) -> Vec<Record> {
    records
        .iter()
        .filter(|record| record.category() == category_filter)
        .cloned()
        .collect()
}

pub fn calculate_average_value(records: &[Record]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }

    let total: f64 = records.iter().map(|record| record.value()).sum();
    total / records.len() as f64
}

pub fn aggregate_by_category(records: &[Record]) -> Vec<(String, f64, usize)> {
    use std::collections::HashMap;

    let mut category_map: HashMap<String, (f64, usize)> = HashMap::new();

    for record in records {
        let entry = category_map
            .entry(record.category().to_string())
            .or_insert((0.0, 0));
        entry.0 += record.value();
        entry.1 += 1;
    }

    category_map
        .into_iter()
        .map(|(category, (total, count))| (category, total, count))
        .collect()
}

pub fn write_results_to_csv(
    results: &[(String, f64, usize)],
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let writer = BufWriter::new(file);
    let mut csv_writer = WriterBuilder::new().from_writer(writer);

    csv_writer.write_record(&["Category", "Total Value", "Record Count"])?;

    for (category, total, count) in results {
        csv_writer.write_record(&[
            category,
            &total.to_string(),
            &count.to_string(),
        ])?;
    }

    csv_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_creation() {
        let record = Record::new(1, "Test".to_string(), "A".to_string(), 100.0, true);
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "Test");
        assert_eq!(record.category, "A");
        assert_eq!(record.value, 100.0);
        assert!(record.active);
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            Record::new(1, "Item1".to_string(), "A".to_string(), 10.0, true),
            Record::new(2, "Item2".to_string(), "B".to_string(), 20.0, true),
            Record::new(3, "Item3".to_string(), "A".to_string(), 30.0, false),
        ];

        let filtered = filter_records_by_category(&records, "A");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category() == "A"));
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            Record::new(1, "Item1".to_string(), "A".to_string(), 10.0, true),
            Record::new(2, "Item2".to_string(), "B".to_string(), 20.0, true),
            Record::new(3, "Item3".to_string(), "A".to_string(), 30.0, true),
        ];

        let avg = calculate_average_value(&records);
        assert_eq!(avg, 20.0);
    }

    #[test]
    fn test_aggregate_by_category() {
        let records = vec![
            Record::new(1, "Item1".to_string(), "A".to_string(), 10.0, true),
            Record::new(2, "Item2".to_string(), "B".to_string(), 20.0, true),
            Record::new(3, "Item3".to_string(), "A".to_string(), 30.0, true),
        ];

        let aggregated = aggregate_by_category(&records);
        assert_eq!(aggregated.len(), 2);

        let a_aggregate = aggregated.iter().find(|(cat, _, _)| cat == "A").unwrap();
        assert_eq!(a_aggregate.1, 40.0);
        assert_eq!(a_aggregate.2, 2);
    }

    #[test]
    fn test_csv_write() -> Result<(), Box<dyn Error>> {
        let results = vec![
            ("A".to_string(), 100.0, 5),
            ("B".to_string(), 200.0, 3),
        ];

        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path().to_str().unwrap();

        write_results_to_csv(&results, temp_path)?;

        let file_content = std::fs::read_to_string(temp_path)?;
        assert!(file_content.contains("Category,Total Value,Record Count"));
        assert!(file_content.contains("A,100,5"));
        assert!(file_content.contains("B,200,3"));

        Ok(())
    }
}