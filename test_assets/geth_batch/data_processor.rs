
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataProcessor {
    file_path: String,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn process(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        let mut records = Vec::new();
        for result in rdr.records() {
            let record = result?;
            let validated_record: Vec<String> = record
                .iter()
                .map(|field| field.trim().to_string())
                .collect();
            
            if !validated_record.is_empty() && validated_record.iter().any(|f| !f.is_empty()) {
                records.push(validated_record);
            }
        }
        
        Ok(records)
    }

    pub fn filter_records(&self, records: Vec<Vec<String>>, column_index: usize, filter_value: &str) -> Vec<Vec<String>> {
        records
            .into_iter()
            .filter(|record| {
                record.get(column_index)
                    .map(|value| value.contains(filter_value))
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn calculate_statistics(&self, records: Vec<Vec<String>>, column_index: usize) -> Result<(f64, f64), Box<dyn Error>> {
        let numeric_values: Vec<f64> = records
            .iter()
            .filter_map(|record| record.get(column_index))
            .filter_map(|value| value.parse::<f64>().ok())
            .collect();

        if numeric_values.is_empty() {
            return Err("No valid numeric data found".into());
        }

        let sum: f64 = numeric_values.iter().sum();
        let count = numeric_values.len() as f64;
        let average = sum / count;

        let variance: f64 = numeric_values
            .iter()
            .map(|value| {
                let diff = average - value;
                diff * diff
            })
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Ok((average, std_dev))
    }
}