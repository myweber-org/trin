
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
            return Err("CSV file contains no data".into());
        }

        Ok(records)
    }

    pub fn transform_numeric_fields(&self, records: &[Vec<String>]) -> Vec<Vec<String>> {
        let mut transformed = Vec::with_capacity(records.len());
        
        for record in records {
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
        records: &[Vec<String>],
        column_index: usize,
        filter_value: &str,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        if records.is_empty() {
            return Ok(Vec::new());
        }

        if column_index >= records[0].len() {
            return Err(format!("Column index {} out of bounds", column_index).into());
        }

        let filtered: Vec<Vec<String>> = records
            .iter()
            .filter(|record| {
                if column_index < record.len() {
                    record[column_index] == filter_value
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        Ok(filtered)
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
        writeln!(temp_file, "Name,Age,Score").unwrap();
        writeln!(temp_file, "Alice,25,95.5").unwrap();
        writeln!(temp_file, "Bob,30,87.25").unwrap();
        writeln!(temp_file, "Charlie,25,91.75").unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor.read_and_validate(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Name", "Age", "Score"]);
        
        let transformed = processor.transform_numeric_fields(&records);
        assert_eq!(transformed[1][2], "95.50");
        
        let filtered = processor.filter_by_column_value(&records, 1, "25").unwrap();
        assert_eq!(filtered.len(), 2);
    }
}