use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_number == 0 && self.has_header {
                continue;
            }

            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|field| field.trim().to_string())
                .collect();

            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), String> {
        if records.is_empty() {
            return Err("No records found".to_string());
        }

        let expected_len = records[0].len();
        for (idx, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(format!(
                    "Record {} has {} fields, expected {}",
                    idx + 1,
                    record.len(),
                    expected_len
                ));
            }
        }

        Ok(())
    }

    pub fn calculate_column_averages(&self, records: &[Vec<String>]) -> Vec<f64> {
        if records.is_empty() {
            return Vec::new();
        }

        let num_columns = records[0].len();
        let mut sums = vec![0.0; num_columns];
        let mut counts = vec![0; num_columns];

        for record in records {
            for (i, field) in record.iter().enumerate() {
                if let Ok(value) = field.parse::<f64>() {
                    sums[i] += value;
                    counts[i] += 1;
                }
            }
        }

        sums.iter()
            .zip(counts.iter())
            .map(|(&sum, &count)| if count > 0 { sum / count as f64 } else { 0.0 })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "col1,col2,col3").unwrap();
        writeln!(temp_file, "1.5,2.0,3.5").unwrap();
        writeln!(temp_file, "4.0,5.5,6.0").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["1.5", "2.0", "3.5"]);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        assert!(processor.validate_records(&records).is_ok());
    }

    #[test]
    fn test_calculate_averages() {
        let records = vec![
            vec!["1.0".to_string(), "2.0".to_string()],
            vec!["3.0".to_string(), "4.0".to_string()],
            vec!["5.0".to_string(), "6.0".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let averages = processor.calculate_column_averages(&records);
        
        assert_eq!(averages.len(), 2);
        assert!((averages[0] - 3.0).abs() < 0.0001);
        assert!((averages[1] - 4.0).abs() < 0.0001);
    }
}