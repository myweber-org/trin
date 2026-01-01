
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 5 {
                let record = CsvRecord {
                    id: parts[0].parse()?,
                    name: parts[1].to_string(),
                    category: parts[2].to_string(),
                    value: parts[3].parse()?,
                    active: parts[4].parse().unwrap_or(false),
                };
                self.records.push(record);
            }
        }
        
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn filter_active(&self) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .cloned()
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.calculate_total_value() / self.records.len() as f64
    }

    pub fn get_records_count(&self) -> usize {
        self.records.len()
    }

    pub fn find_max_value_record(&self) -> Option<CsvRecord> {
        self.records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
            .cloned()
    }

    pub fn find_min_value_record(&self) -> Option<CsvRecord> {
        self.records
            .iter()
            .min_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
            .cloned()
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<CsvRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record.clone());
        }
        
        groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,category,value,active").unwrap();
        writeln!(temp_file, "1,ItemA,Electronics,150.5,true").unwrap();
        writeln!(temp_file, "2,ItemB,Books,25.0,true").unwrap();
        writeln!(temp_file, "3,ItemC,Electronics,300.0,false").unwrap();
        
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(processor.get_records_count(), 3);
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let active_items = processor.filter_active();
        assert_eq!(active_items.len(), 2);
        
        let total_value = processor.calculate_total_value();
        assert_eq!(total_value, 475.5);
        
        let average_value = processor.calculate_average_value();
        assert_eq!(average_value, 158.5);
        
        let max_record = processor.find_max_value_record();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().value, 300.0);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("Electronics").unwrap().len(), 2);
        assert_eq!(groups.get("Books").unwrap().len(), 1);
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn load_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)?;
    
    let mut records = Vec::new();
    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    
    Ok(records)
}

fn filter_records(records: &[Record], category_filter: &str) -> Vec<Record> {
    records
        .iter()
        .filter(|r| r.category == category_filter && r.active)
        .cloned()
        .collect()
}

fn aggregate_values(records: &[Record]) -> (f64, f64, usize) {
    let total: f64 = records.iter().map(|r| r.value).sum();
    let avg = if !records.is_empty() {
        total / records.len() as f64
    } else {
        0.0
    };
    let count = records.len();
    
    (total, avg, count)
}

fn save_results<P: AsRef<Path>>(records: &[Record], path: P) -> Result<(), Box<dyn Error>> {
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_path(path)?;
    
    for record in records {
        writer.serialize(record)?;
    }
    
    writer.flush()?;
    Ok(())
}

fn process_data(input_path: &str, output_path: &str, category: &str) -> Result<(), Box<dyn Error>> {
    let all_records = load_csv(input_path)?;
    let filtered_records = filter_records(&all_records, category);
    
    let (total, average, count) = aggregate_values(&filtered_records);
    println!("Processed {} records", count);
    println!("Total value: {:.2}", total);
    println!("Average value: {:.2}", average);
    
    if !filtered_records.is_empty() {
        save_results(&filtered_records, output_path)?;
        println!("Results saved to {}", output_path);
    } else {
        println!("No records matched the filter criteria");
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";
    let target_category = "electronics";
    
    process_data(input_file, output_file, target_category)
}