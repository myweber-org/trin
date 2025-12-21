use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

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
                return Err(format!("Empty line found at line {}", line_number).into());
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err("CSV file is empty".into());
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
            let mut new_record = record.clone();

            if i >= start_index && column_index < record.len() {
                if let Ok(value) = record[column_index].parse::<f64>() {
                    let transformed_value = value * multiplier;
                    new_record[column_index] = transformed_value.to_string();
                } else {
                    return Err(format!(
                        "Non-numeric value in column {} at record {}",
                        column_index, i
                    )
                    .into());
                }
            }

            transformed.push(new_record);
        }

        Ok(transformed)
    }

    pub fn write_to_file(&self, records: &[Vec<String>], output_path: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(output_path)?;

        for record in records {
            let line = record.join(&self.delimiter.to_string());
            writeln!(file, "{}", line)?;
        }

        Ok(())
    }

    pub fn calculate_column_stats(&self, records: &[Vec<String>], column_index: usize) -> Result<(f64, f64, f64), Box<dyn Error>> {
        let start_index = if self.has_headers { 1 } else { 0 };
        let mut values = Vec::new();
        let mut sum = 0.0;

        for (i, record) in records.iter().enumerate() {
            if i >= start_index && column_index < record.len() {
                match record[column_index].parse::<f64>() {
                    Ok(value) => {
                        values.push(value);
                        sum += value;
                    }
                    Err(_) => return Err(format!("Invalid numeric value at record {} column {}", i, column_index).into()),
                }
            }
        }

        if values.is_empty() {
            return Err("No valid numeric values found in specified column".into());
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let count = values.len() as f64;
        let mean = sum / count;
        let median = if count as usize % 2 == 0 {
            (values[count as usize / 2 - 1] + values[count as usize / 2]) / 2.0
        } else {
            values[count as usize / 2]
        };

        let variance: f64 = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / count;
        let std_dev = variance.sqrt();

        Ok((mean, median, std_dev))
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
        let records = processor.read_and_validate(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 4);

        let transformed = processor.transform_numeric_fields(&records, 2, 1.1).unwrap();
        assert_eq!(transformed[1][2], "55000");
        assert_eq!(transformed[2][2], "49500");

        let stats = processor.calculate_column_stats(&records, 1).unwrap();
        assert!((stats.0 - 30.0).abs() < 0.001);
    }
}