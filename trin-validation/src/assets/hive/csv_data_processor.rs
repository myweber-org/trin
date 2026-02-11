use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

#[derive(Debug, Clone)]
struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    fn new(id: u32, category: String, value: f64, active: bool) -> Self {
        Self { id, category, value, active }
    }
}

fn read_csv_data(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: DataRecord = result?;
        records.push(record);
    }

    Ok(records)
}

fn filter_active_records(records: &[DataRecord]) -> Vec<DataRecord> {
    records.iter()
        .filter(|record| record.active)
        .cloned()
        .collect()
}

fn calculate_category_averages(records: &[DataRecord]) -> Vec<(String, f64)> {
    use std::collections::HashMap;
    
    let mut category_sums: HashMap<String, (f64, usize)> = HashMap::new();
    
    for record in records {
        let entry = category_sums.entry(record.category.clone())
            .or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }
    
    category_sums.iter()
        .map(|(category, (sum, count))| (category.clone(), sum / *count as f64))
        .collect()
}

fn write_processed_data(output_path: &str, averages: &[(String, f64)]) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut writer = Writer::from_writer(file);
    
    writer.write_record(&["Category", "AverageValue"])?;
    
    for (category, average) in averages {
        writer.write_record(&[category, &average.to_string()])?;
    }
    
    writer.flush()?;
    Ok(())
}

fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let records = read_csv_data(input_path)?;
    let active_records = filter_active_records(&records);
    let category_averages = calculate_category_averages(&active_records);
    write_processed_data(output_path, &category_averages)?;
    
    println!("Processed {} records", records.len());
    println!("Found {} active records", active_records.len());
    println!("Generated averages for {} categories", category_averages.len());
    
    Ok(())
}

fn main() {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";
    
    match process_csv_data(input_file, output_file) {
        Ok(()) => println!("Data processing completed successfully"),
        Err(e) => eprintln!("Error processing data: {}", e),
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
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

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
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
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let name = parts[1].to_string();
            
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let category = parts[3].to_string();
            
            self.records.push(CsvRecord {
                id,
                name,
                value,
                category,
            });
            
            count += 1;
        }
        
        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
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

    pub fn transform_values<F>(&mut self, transform_fn: F)
    where
        F: Fn(f64) -> f64,
    {
        for record in &mut self.records {
            record.value = transform_fn(record.value);
        }
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_processor() {
        let processor = CsvProcessor::new();
        assert_eq!(processor.get_records().len(), 0);
        assert_eq!(processor.calculate_average(), None);
    }

    #[test]
    fn test_filter_records() {
        let mut processor = CsvProcessor::new();
        
        processor.records.push(CsvRecord {
            id: 1,
            name: "Test1".to_string(),
            value: 10.5,
            category: "A".to_string(),
        });
        
        processor.records.push(CsvRecord {
            id: 2,
            name: "Test2".to_string(),
            value: 20.0,
            category: "B".to_string(),
        });
        
        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }

    #[test]
    fn test_value_transformation() {
        let mut processor = CsvProcessor::new();
        
        processor.records.push(CsvRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.0,
            category: "Test".to_string(),
        });
        
        processor.transform_values(|x| x * 2.0);
        assert_eq!(processor.records[0].value, 20.0);
    }
}