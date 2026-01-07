use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = match lines.next() {
            Some(header_line) => header_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            None => return Err("Empty CSV file".into()),
        };

        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            if line.trim().is_empty() {
                continue;
            }
            let record: Vec<String> = line
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                records.push(record);
            } else {
                eprintln!("Skipping malformed record: {}", line);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column<F>(&self, column_name: &str, predicate: F) -> Result<Vec<Vec<String>>, Box<dyn Error>>
    where
        F: Fn(&str) -> bool,
    {
        let col_index = self.headers
            .iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;

        let filtered: Vec<Vec<String>> = self.records
            .iter()
            .filter(|record| predicate(&record[col_index]))
            .cloned()
            .collect();

        Ok(filtered)
    }

    pub fn get_column_summary(&self, column_name: &str) -> Result<(usize, String, String), Box<dyn Error>> {
        let col_index = self.headers
            .iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;

        let values: Vec<&str> = self.records
            .iter()
            .map(|record| record[col_index].as_str())
            .collect();

        let count = values.len();
        let unique_count = values.iter().collect::<std::collections::HashSet<_>>().len();
        let sample_value = values.first().unwrap_or(&"").to_string();

        Ok((count, unique_count.to_string(), sample_value))
    }

    pub fn save_filtered<P: AsRef<Path>>(&self, filtered_records: &[Vec<String>], output_path: P) -> Result<(), Box<dyn Error>> {
        let mut writer = csv::Writer::from_path(output_path)?;
        writer.write_record(&self.headers)?;
        for record in filtered_records {
            writer.write_record(record)?;
        }
        writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,age,active").unwrap();
        writeln!(file, "1,Alice,30,true").unwrap();
        writeln!(file, "2,Bob,25,false").unwrap();
        writeln!(file, "3,Charlie,35,true").unwrap();
        writeln!(file, "4,Diana,28,true").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path()).unwrap();
        assert_eq!(processor.headers, vec!["id", "name", "age", "active"]);
        assert_eq!(processor.records.len(), 4);
    }

    #[test]
    fn test_filter_by_column() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path()).unwrap();
        let filtered = processor.filter_by_column("active", |val| val == "true").unwrap();
        assert_eq!(filtered.len(), 3);
        assert_eq!(filtered[0][1], "Alice");
    }

    #[test]
    fn test_column_summary() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path()).unwrap();
        let (count, unique, sample) = processor.get_column_summary("age").unwrap();
        assert_eq!(count, 4);
        assert_eq!(unique, "4");
        assert_eq!(sample, "30");
    }
}