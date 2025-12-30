use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = if let Some(first_line) = lines.next() {
            first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            return Err("Empty CSV file".into());
        };

        let mut records = Vec::new();
        for line in lines {
            let line = line?;
            let record: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, predicate: impl Fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = self.headers.iter().position(|h| h == column_name);
        
        column_index.map_or_else(Vec::new, |idx| {
            self.records
                .iter()
                .filter(|record| predicate(&record[idx]))
                .cloned()
                .collect()
        })
    }

    pub fn aggregate_numeric_column(&self, column_name: &str, operation: &str) -> Option<f64> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let numeric_values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record[column_index].parse().ok())
            .collect();

        if numeric_values.is_empty() {
            return None;
        }

        match operation {
            "sum" => Some(numeric_values.iter().sum()),
            "avg" => Some(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64),
            "min" => numeric_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            "max" => numeric_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            _ => None,
        }
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(path)?;
        
        writeln!(file, "{}", self.headers.join(","))?;
        
        for record in &self.records {
            writeln!(file, "{}", record.join(","))?;
        }
        
        Ok(())
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }
}

pub fn process_csv_file(input_path: &str, output_path: &str, filter_column: &str, filter_value: &str) -> Result<(), Box<dyn Error>> {
    let processor = CsvProcessor::from_file(input_path)?;
    
    println!("Loaded {} records with columns: {:?}", processor.get_record_count(), processor.get_headers());
    
    let filtered = processor.filter_by_column(filter_column, |value| value == filter_value);
    
    if filtered.is_empty() {
        println!("No records match filter criteria");
        return Ok(());
    }
    
    let mut filtered_processor = CsvProcessor {
        headers: processor.headers.clone(),
        records: filtered,
    };
    
    filtered_processor.write_to_file(output_path)?;
    println!("Filtered data written to {}", output_path);
    
    Ok(())
}