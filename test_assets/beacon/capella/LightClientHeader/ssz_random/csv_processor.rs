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

    pub fn filter_rows<F>(&self, predicate: F) -> Result<Vec<Vec<String>>, Box<dyn Error>>
    where
        F: Fn(&[String]) -> bool,
    {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut filtered_data = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let columns: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if predicate(&columns) {
                filtered_data.push(columns);
            }
        }

        Ok(filtered_data)
    }

    pub fn get_column_stats(&self, column_index: usize) -> Result<(f64, f64, f64), Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut values = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let columns: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if column_index < columns.len() {
                if let Ok(value) = columns[column_index].parse::<f64>() {
                    values.push(value);
                }
            }
        }

        if values.is_empty() {
            return Err("No valid numeric data found".into());
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        Ok((mean, variance, std_dev))
    }
}

pub fn process_csv_data() -> Result<(), Box<dyn Error>> {
    let processor = CsvProcessor::new("data.csv", ',');
    
    let filtered = processor.filter_rows(|columns| {
        columns.len() >= 3 && !columns[2].is_empty()
    })?;

    println!("Filtered rows: {}", filtered.len());

    if !filtered.is_empty() {
        let stats = processor.get_column_stats(1)?;
        println!("Column statistics - Mean: {:.2}, Variance: {:.2}, Std Dev: {:.2}", 
                 stats.0, stats.1, stats.2);
    }

    Ok(())
}