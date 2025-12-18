
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub columns: Vec<String>,
}

impl CsvRecord {
    pub fn new(data: Vec<String>) -> Self {
        CsvRecord { columns: data }
    }

    pub fn validate(&self, expected_columns: usize) -> Result<(), String> {
        if self.columns.len() != expected_columns {
            return Err(format!(
                "Expected {} columns, found {}",
                expected_columns,
                self.columns.len()
            ));
        }
        Ok(())
    }

    pub fn transform_numeric(&self, column_index: usize, multiplier: f64) -> Result<f64, String> {
        let value = self
            .columns
            .get(column_index)
            .ok_or_else(|| format!("Column index {} out of bounds", column_index))?;

        let parsed = value
            .parse::<f64>()
            .map_err(|_| format!("Failed to parse '{}' as numeric", value))?;

        Ok(parsed * multiplier)
    }
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
    delimiter: char,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
            delimiter: ',',
        }
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let columns: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            self.records.push(CsvRecord::new(columns));
        }

        Ok(())
    }

    pub fn process_records<F>(&self, mut processor: F) -> Result<(), Box<dyn Error>>
    where
        F: FnMut(&CsvRecord) -> Result<(), Box<dyn Error>>,
    {
        for record in &self.records {
            processor(record)?;
        }
        Ok(())
    }

    pub fn filter_records<F>(&self, predicate: F) -> Vec<CsvRecord>
    where
        F: Fn(&CsvRecord) -> bool,
    {
        self.records
            .iter()
            .filter(|record| predicate(record))
            .cloned()
            .collect()
    }

    pub fn calculate_column_average(&self, column_index: usize) -> Result<f64, String> {
        let mut sum = 0.0;
        let mut count = 0;

        for record in &self.records {
            match record.transform_numeric(column_index, 1.0) {
                Ok(value) => {
                    sum += value;
                    count += 1;
                }
                Err(_) => continue,
            }
        }

        if count == 0 {
            return Err("No valid numeric values found in column".to_string());
        }

        Ok(sum / count as f64)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_record_validation() {
        let record = CsvRecord::new(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        assert!(record.validate(3).is_ok());
        assert!(record.validate(2).is_err());
    }

    #[test]
    fn test_numeric_transformation() {
        let record = CsvRecord::new(vec!["10.5".to_string(), "text".to_string()]);
        assert_eq!(record.transform_numeric(0, 2.0).unwrap(), 21.0);
        assert!(record.transform_numeric(1, 1.0).is_err());
    }

    #[test]
    fn test_csv_processing() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "name,age,score")?;
        writeln!(temp_file, "Alice,25,95.5")?;
        writeln!(temp_file, "Bob,30,88.0")?;
        writeln!(temp_file, "Charlie,35,92.5")?;

        let mut processor = CsvProcessor::new();
        processor.load_from_file(temp_file.path())?;

        assert_eq!(processor.record_count(), 3);

        let average_score = processor.calculate_column_average(2)?;
        assert!((average_score - 92.0).abs() < 0.01);

        let filtered = processor.filter_records(|record| {
            record
                .transform_numeric(1, 1.0)
                .map(|age| age > 30.0)
                .unwrap_or(false)
        });
        assert_eq!(filtered.len(), 1);

        Ok(())
    }
}