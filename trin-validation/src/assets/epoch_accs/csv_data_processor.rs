use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    file_path: String,
    delimiter: char,
}

impl CsvProcessor {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        CsvProcessor {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn read_and_filter(&self, column_index: usize, filter_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut filtered_records = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let record: Vec<String> = line.split(self.delimiter).map(|s| s.to_string()).collect();
            
            if record.len() > column_index && record[column_index] == filter_value {
                filtered_records.push(record);
            }
        }

        Ok(filtered_records)
    }

    pub fn count_records(&self) -> Result<usize, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let count = reader.lines().count();
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,age").unwrap();
        writeln!(temp_file, "1,alice,30").unwrap();
        writeln!(temp_file, "2,bob,25").unwrap();
        writeln!(temp_file, "3,alice,28").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let filtered = processor.read_and_filter(1, "alice").unwrap();
        
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][0], "1");
        assert_eq!(filtered[1][0], "3");
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

fn load_records(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
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

fn write_processed_data(
    output_path: &str,
    active_records: &[&Record],
    averages: &[(String, f64)],
) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(output_path)?;

    writer.write_record(&["ID", "Category", "Value", "Status"])?;
    for record in active_records {
        writer.serialize((
            record.id,
            &record.category,
            record.value,
            "ACTIVE",
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

pub fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let records = load_records(input_path)?;
    let active_records = filter_active_records(&records);
    let category_averages = calculate_category_averages(&records);

    write_processed_data(output_path, &active_records, &category_averages)?;

    println!("Processed {} records", records.len());
    println!("Found {} active records", active_records.len());
    println!("Calculated averages for {} categories", category_averages.len());

    Ok(())
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_active_records(&self) -> Vec<&Record> {
        self.records.iter().filter(|r| r.active).collect()
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn count_by_category(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        
        for record in &self.records {
            *counts.entry(record.category.clone()).or_insert(0) += 1;
        }
        
        counts
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
        writeln!(temp_file, "id,category,value,active").unwrap();
        writeln!(temp_file, "1,electronics,299.99,true").unwrap();
        writeln!(temp_file, "2,books,19.99,true").unwrap();
        writeln!(temp_file, "3,electronics,599.99,false").unwrap();
        
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        
        let active_records = processor.get_active_records();
        assert_eq!(active_records.len(), 2);
        
        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().value, 599.99);
        
        let counts = processor.count_by_category();
        assert_eq!(counts.get("electronics"), Some(&2));
        assert_eq!(counts.get("books"), Some(&1));
    }
}