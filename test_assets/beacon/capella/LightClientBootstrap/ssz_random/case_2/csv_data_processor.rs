
use std::error::Error;
use std::fs::File;
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, category: String, value: f64, timestamp: String) -> Self {
        DataRecord {
            id,
            category,
            value,
            timestamp,
        }
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

        for result in rdr.records() {
            let record = result?;
            let id: u32 = record[0].parse()?;
            let category = record[1].to_string();
            let value: f64 = record[2].parse()?;
            let timestamp = record[3].to_string();

            self.records.push(DataRecord::new(id, category, value, timestamp));
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    pub fn export_filtered_csv(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let file = File::create(output_path)?;
        let mut wtr = WriterBuilder::new().from_writer(file);

        wtr.write_record(&["id", "category", "value", "timestamp"])?;

        for record in filtered {
            wtr.write_record(&[
                record.id.to_string(),
                record.category.clone(),
                record.value.to_string(),
                record.timestamp.clone(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average();

        (min, max, avg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,category,value,timestamp").unwrap();
        writeln!(temp_file, "1,electronics,250.5,2024-01-15").unwrap();
        writeln!(temp_file, "2,books,45.99,2024-01-16").unwrap();
        writeln!(temp_file, "3,electronics,189.75,2024-01-17").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let avg = processor.calculate_average();
        assert!((avg - 162.08).abs() < 0.01);
        
        let (min, max, avg_stat) = processor.get_statistics();
        assert_eq!(min, 45.99);
        assert_eq!(max, 250.5);
        assert!((avg_stat - 162.08).abs() < 0.01);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub fn load_csv_data(file_path: &str) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 4 {
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].to_string();

            records.push(CsvRecord {
                id,
                name,
                value,
                category,
            });
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[CsvRecord], category: &str) -> Vec<&CsvRecord> {
    records
        .iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_average(records: &[&CsvRecord]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|record| record.value).sum();
    Some(sum / records.len() as f64)
}

pub fn find_max_value(records: &[CsvRecord]) -> Option<&CsvRecord> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

pub fn aggregate_by_category(records: &[CsvRecord]) -> Vec<(String, f64)> {
    use std::collections::HashMap;

    let mut category_totals: HashMap<String, f64> = HashMap::new();

    for record in records {
        *category_totals.entry(record.category.clone()).or_insert(0.0) += record.value;
    }

    let mut result: Vec<(String, f64)> = category_totals.into_iter().collect();
    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,value,category").unwrap();
        writeln!(file, "1,ItemA,100.5,Electronics").unwrap();
        writeln!(file, "2,ItemB,75.2,Books").unwrap();
        writeln!(file, "3,ItemC,150.0,Electronics").unwrap();
        writeln!(file, "4,ItemD,50.8,Books").unwrap();
        file
    }

    #[test]
    fn test_load_csv_data() {
        let test_file = create_test_csv();
        let records = load_csv_data(test_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 4);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].category, "Books");
    }

    #[test]
    fn test_filter_by_category() {
        let test_file = create_test_csv();
        let records = load_csv_data(test_file.path().to_str().unwrap()).unwrap();
        let electronics = filter_by_category(&records, "Electronics");
        assert_eq!(electronics.len(), 2);
    }

    #[test]
    fn test_calculate_average() {
        let test_file = create_test_csv();
        let records = load_csv_data(test_file.path().to_str().unwrap()).unwrap();
        let electronics = filter_by_category(&records, "Electronics");
        let avg = calculate_average(&electronics).unwrap();
        assert!((avg - 125.25).abs() < 0.01);
    }

    #[test]
    fn test_find_max_value() {
        let test_file = create_test_csv();
        let records = load_csv_data(test_file.path().to_str().unwrap()).unwrap();
        let max_record = find_max_value(&records).unwrap();
        assert_eq!(max_record.id, 3);
        assert_eq!(max_record.value, 150.0);
    }

    #[test]
    fn test_aggregate_by_category() {
        let test_file = create_test_csv();
        let records = load_csv_data(test_file.path().to_str().unwrap()).unwrap();
        let aggregates = aggregate_by_category(&records);
        assert_eq!(aggregates.len(), 2);
        assert_eq!(aggregates[0].0, "Electronics");
        assert_eq!(aggregates[0].1, 250.5);
    }
}