use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u8,
    active: bool,
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    fn filter_by_age(&self, min_age: u8, max_age: u8) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.age >= min_age && record.age <= max_age)
            .collect()
    }

    fn save_to_csv(&self, file_path: &str, records: Vec<&Record>) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let mut wtr = Writer::from_writer(file);

        for record in records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    fn calculate_average_age(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let total: u32 = self.records.iter().map(|r| r.age as u32).sum();
        total as f64 / self.records.len() as f64
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_csv("input.csv")?;
    
    println!("Total records loaded: {}", processor.records.len());
    println!("Average age: {:.2}", processor.calculate_average_age());
    
    let active_records = processor.filter_active();
    println!("Active records: {}", active_records.len());
    
    processor.save_to_csv("active_records.csv", active_records)?;
    
    let age_filtered = processor.filter_by_age(25, 40);
    println!("Records between 25-40: {}", age_filtered.len());
    
    processor.save_to_csv("age_filtered.csv", age_filtered)?;
    
    Ok(())
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: String,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: String, category: String) -> Self {
        DataRecord {
            id,
            value,
            timestamp,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.timestamp.is_empty() && 
        !self.category.is_empty() && 
        self.value.is_finite()
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }
            
            let id = match parts[0].parse::<u32>() {
                Ok(id) => id,
                Err(_) => continue,
            };
            
            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };
            
            let record = DataRecord::new(
                id,
                value,
                parts[2].to_string(),
                parts[3].to_string(),
            );
            
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }
        
        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn get_statistics(&self) -> DataStatistics {
        let count = self.records.len();
        let avg = self.calculate_average().unwrap_or(0.0);
        
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let variance: f64 = if count > 1 {
            values.iter().map(|&v| (v - avg).powi(2)).sum::<f64>() / (count - 1) as f64
        } else {
            0.0
        };
        
        DataStatistics {
            count,
            average: avg,
            variance,
            min_value: self.records.iter().map(|r| r.value).fold(f64::INFINITY, f64::min),
            max_value: self.records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max),
        }
    }

    pub fn export_valid_records<P: AsRef<Path>>(&self, path: P) -> Result<usize, Box<dyn Error>> {
        let mut valid_count = 0;
        let mut output = String::new();
        output.push_str("id,value,timestamp,category\n");
        
        for record in &self.records {
            if record.is_valid() {
                output.push_str(&format!("{},{},{},{}\n", 
                    record.id, 
                    record.value, 
                    record.timestamp, 
                    record.category));
                valid_count += 1;
            }
        }
        
        std::fs::write(path, output)?;
        Ok(valid_count)
    }
}

#[derive(Debug)]
pub struct DataStatistics {
    pub count: usize,
    pub average: f64,
    pub variance: f64,
    pub min_value: f64,
    pub max_value: f64,
}

impl std::fmt::Display for DataStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Statistics: Count={}, Avg={:.2}, Var={:.2}, Min={:.2}, Max={:.2}",
            self.count, self.average, self.variance, self.min_value, self.max_value)
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.trim().is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self {
            id,
            value,
            category: category.trim().to_string(),
        })
    }

    pub fn calculate_tax(&self, rate: f64) -> f64 {
        self.value * rate
    }
}

pub fn load_records_from_file(path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid format at line {}", line_num + 1).into());
        }

        let id = parts[0].parse::<u32>()?;
        let value = parts[1].parse::<f64>()?;
        let category = parts[2].to_string();

        match DataRecord::new(id, value, category) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Warning: Skipping line {}: {}", line_num + 1, e),
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[DataRecord], category: &str) -> Vec<&DataRecord> {
    records
        .iter()
        .filter(|r| r.category == category)
        .collect()
}

pub fn calculate_total(records: &[DataRecord]) -> f64 {
    records.iter().map(|r| r.value).sum()
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() {
                continue;
            }

            if self.has_header && line_number == 0 {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !self.validate_record(&fields) {
                return Err(format!("Invalid record at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        Ok(records)
    }

    fn validate_record(&self, fields: &[String]) -> bool {
        !fields.is_empty() && fields.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, data: &[Vec<String>], column_index: usize) -> Result<(f64, f64), Box<dyn Error>> {
        if data.is_empty() {
            return Err("No data available for statistics".into());
        }

        let mut values = Vec::new();
        for record in data {
            if column_index >= record.len() {
                return Err(format!("Column index {} out of bounds", column_index).into());
            }

            match record[column_index].parse::<f64>() {
                Ok(value) => values.push(value),
                Err(_) => return Err(format!("Invalid numeric value at column {}", column_index).into()),
            }
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        let variance: f64 = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        Ok((mean, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.0").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.len(), 3);
        assert_eq!(data[0], vec!["Alice", "30", "50000.0"]);
    }

    #[test]
    fn test_statistics_calculation() {
        let data = vec![
            vec!["50000.0".to_string()],
            vec!["45000.0".to_string()],
            vec!["55000.0".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let stats = processor.calculate_statistics(&data, 0);
        
        assert!(stats.is_ok());
        let (mean, std_dev) = stats.unwrap();
        assert!((mean - 50000.0).abs() < 0.01);
        assert!(std_dev > 0.0);
    }
}