
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            headers: Vec::new(),
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(header_line) = lines.next() {
            self.headers = header_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        for line in lines {
            let record: Vec<String> = line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == self.headers.len() {
                self.records.push(record);
            }
        }

        Ok(())
    }

    pub fn filter_by_column_value(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(index) => index,
            None => return Vec::new(),
        };

        self.records
            .iter()
            .filter(|record| record.get(column_index) == Some(&value.to_string()))
            .cloned()
            .collect()
    }

    pub fn aggregate_numeric_column(&self, column_name: &str) -> Option<f64> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;

        let sum: f64 = self.records
            .iter()
            .filter_map(|record| record.get(column_index))
            .filter_map(|value| value.parse::<f64>().ok())
            .sum();

        Some(sum)
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_headers(&self) -> &Vec<String> {
        &self.headers
    }
}

pub fn process_csv_data(file_path: &str, filter_column: &str, filter_value: &str, aggregate_column: &str) -> Result<(), Box<dyn Error>> {
    let mut processor = CsvProcessor::new();
    processor.load_from_file(file_path)?;

    println!("Loaded {} records", processor.get_record_count());
    println!("Headers: {:?}", processor.get_headers());

    let filtered = processor.filter_by_column_value(filter_column, filter_value);
    println!("Filtered records matching {} = {}: {}", filter_column, filter_value, filtered.len());

    if let Some(total) = processor.aggregate_numeric_column(aggregate_column) {
        println!("Total for column {}: {:.2}", aggregate_column, total);
    } else {
        println!("Could not aggregate column {}", aggregate_column);
    }

    Ok(())
}