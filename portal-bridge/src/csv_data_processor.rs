use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
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

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            if index == 0 {
                continue;
            }
            
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() >= 4 {
                let record = CsvRecord {
                    id: parts[0].parse()?,
                    name: parts[1].to_string(),
                    value: parts[2].parse()?,
                    category: parts[3].to_string(),
                };
                self.records.push(record);
            }
        }
        
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&CsvRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let count = values.len() as f64;
        
        if count == 0.0 {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = values.iter().sum();
        let average = sum / count;
        
        let variance: f64 = values.iter()
            .map(|value| {
                let diff = average - value;
                diff * diff
            })
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (average, variance, std_dev)
    }
}

impl Default for CsvProcessor {
    fn default() -> Self {
        Self::new()
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
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,100.5,Electronics").unwrap();
        writeln!(temp_file, "2,ItemB,75.2,Books").unwrap();
        writeln!(temp_file, "3,ItemC,150.0,Electronics").unwrap();
        
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let avg = processor.calculate_average();
        assert!((avg - 108.56666666666666).abs() < 0.0001);
        
        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.id, 3);
        assert_eq!(max_record.value, 150.0);
    }
}use std::error::Error;
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
            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.is_empty() {
                return Err(format!("Empty line at line {}", line_number).into());
            }

            if fields.iter().any(|field| field.is_empty()) {
                return Err(format!("Empty field detected at line {}", line_number).into());
            }

            records.push(fields);
        }

        if self.has_headers && !records.is_empty() {
            records.remove(0);
        }

        Ok(records)
    }

    pub fn transform_numeric_fields(
        &self,
        data: &[Vec<String>],
        column_index: usize,
        multiplier: f64,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let mut transformed = Vec::with_capacity(data.len());

        for (row_index, row) in data.iter().enumerate() {
            if column_index >= row.len() {
                return Err(format!(
                    "Column index {} out of bounds at row {}",
                    column_index,
                    row_index + 1
                )
                .into());
            }

            let mut new_row = row.clone();
            let value = &row[column_index];

            match value.parse::<f64>() {
                Ok(num) => {
                    let transformed_value = (num * multiplier).to_string();
                    new_row[column_index] = transformed_value;
                }
                Err(_) => {
                    return Err(format!(
                        "Non-numeric value '{}' at row {}, column {}",
                        value,
                        row_index + 1,
                        column_index
                    )
                    .into());
                }
            }

            transformed.push(new_row);
        }

        Ok(transformed)
    }

    pub fn filter_records(
        &self,
        data: &[Vec<String>],
        column_index: usize,
        predicate: impl Fn(&str) -> bool,
    ) -> Vec<Vec<String>> {
        data.iter()
            .filter(|row| column_index < row.len() && predicate(&row[column_index]))
            .cloned()
            .collect()
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
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,85.5").unwrap();
        writeln!(temp_file, "Bob,30,92.0").unwrap();
        writeln!(temp_file, "Charlie,35,78.5").unwrap();

        let processor = CsvProcessor::new(',', true);
        let data = processor.read_and_validate(temp_file.path().to_str().unwrap());
        assert!(data.is_ok());

        let records = data.unwrap();
        assert_eq!(records.len(), 3);

        let transformed = processor.transform_numeric_fields(&records, 2, 1.1);
        assert!(transformed.is_ok());

        let filtered = processor.filter_records(&records, 1, |age| age.parse::<i32>().unwrap() > 28);
        assert_eq!(filtered.len(), 2);
    }
}