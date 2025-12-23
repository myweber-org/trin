
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
            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.is_empty() {
                return Err(format!("Empty line at line {}", line_number).into());
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err("File contains no data".into());
        }

        Ok(records)
    }

    pub fn transform_numeric_fields(
        &self,
        records: &[Vec<String>],
        column_index: usize,
        multiplier: f64,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let mut transformed = Vec::new();
        let start_index = if self.has_headers { 1 } else { 0 };

        for (i, record) in records.iter().enumerate() {
            if i == 0 && self.has_headers {
                transformed.push(record.clone());
                continue;
            }

            if column_index >= record.len() {
                return Err(format!(
                    "Column index {} out of bounds for record at line {}",
                    column_index,
                    i + 1
                )
                .into());
            }

            let mut new_record = record.clone();
            match record[column_index].parse::<f64>() {
                Ok(value) => {
                    let transformed_value = value * multiplier;
                    new_record[column_index] = transformed_value.to_string();
                }
                Err(_) => {
                    return Err(format!(
                        "Non-numeric value in column {} at line {}",
                        column_index,
                        i + 1
                    )
                    .into());
                }
            }
            transformed.push(new_record);
        }

        Ok(transformed)
    }

    pub fn filter_records(
        &self,
        records: &[Vec<String>],
        predicate: impl Fn(&[String]) -> bool,
    ) -> Vec<Vec<String>> {
        let start_index = if self.has_headers { 1 } else { 0 };
        let mut filtered = Vec::new();

        if self.has_headers && !records.is_empty() {
            filtered.push(records[0].clone());
        }

        for record in records.iter().skip(start_index) {
            if predicate(record) {
                filtered.push(record.clone());
            }
        }

        filtered
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
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,60000").unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor
            .read_and_validate(temp_file.path().to_str().unwrap())
            .unwrap();

        assert_eq!(records.len(), 4);
        assert_eq!(records[0], vec!["name", "age", "salary"]);

        let transformed = processor
            .transform_numeric_fields(&records, 2, 1.1)
            .unwrap();
        assert_eq!(transformed[1][2], "55000");
        assert_eq!(transformed[2][2], "49500");
        assert_eq!(transformed[3][2], "66000");

        let filtered = processor.filter_records(&records, |record| {
            record[1].parse::<i32>().unwrap_or(0) > 30
        });
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[1][0], "Charlie");
    }
}