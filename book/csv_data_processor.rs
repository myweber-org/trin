use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    delimiter: char,
    has_headers: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_headers: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_headers,
        }
    }

    pub fn read_and_validate(&self, file_path: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line?;
            
            if line_content.trim().is_empty() {
                continue;
            }

            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.is_empty() {
                return Err(format!("Empty record at line {}", line_number).into());
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err("File contains no valid data".into());
        }

        Ok(records)
    }

    pub fn transform_numeric_fields(&self, data: &[Vec<String>]) -> Vec<Vec<String>> {
        let mut transformed = Vec::with_capacity(data.len());

        for record in data {
            let transformed_record: Vec<String> = record
                .iter()
                .map(|field| {
                    if let Ok(num) = field.parse::<f64>() {
                        format!("{:.2}", num)
                    } else {
                        field.clone()
                    }
                })
                .collect();
            transformed.push(transformed_record);
        }

        transformed
    }

    pub fn filter_by_column_value(
        &self,
        data: &[Vec<String>],
        column_index: usize,
        filter_value: &str,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        if column_index >= data[0].len() {
            return Err("Column index out of bounds".into());
        }

        let filtered: Vec<Vec<String>> = data
            .iter()
            .filter(|record| record[column_index] == filter_value)
            .cloned()
            .collect();

        Ok(filtered)
    }
}

pub fn calculate_column_average(data: &[Vec<String>], column_index: usize) -> Result<f64, Box<dyn Error>> {
    if data.is_empty() {
        return Err("No data available for calculation".into());
    }

    if column_index >= data[0].len() {
        return Err("Column index out of bounds".into());
    }

    let mut sum = 0.0;
    let mut count = 0;

    for record in data {
        if let Ok(value) = record[column_index].parse::<f64>() {
            sum += value;
            count += 1;
        }
    }

    if count == 0 {
        return Err("No numeric values found in specified column".into());
    }

    Ok(sum / count as f64)
}