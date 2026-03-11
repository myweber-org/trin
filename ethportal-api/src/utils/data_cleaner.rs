
use std::collections::HashSet;

pub fn clean_string_data(input: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for item in input {
        let normalized = item.trim().to_lowercase();
        if !normalized.is_empty() && seen.insert(normalized.clone()) {
            result.push(normalized);
        }
    }

    result.sort();
    result
}

pub fn remove_numeric_duplicates(input: Vec<f64>) -> Vec<f64> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for &num in &input {
        if seen.insert((num * 1000.0).round() as i64) {
            result.push(num);
        }
    }

    result.sort_by(|a, b| a.partial_cmp(b).unwrap());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_string_data() {
        let input = vec![
            "  Apple  ".to_string(),
            "apple".to_string(),
            "BANANA".to_string(),
            "banana ".to_string(),
            "".to_string(),
            "  Cherry  ".to_string(),
        ];
        
        let result = clean_string_data(input);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_remove_numeric_duplicates() {
        let input = vec![3.141, 3.1415, 2.718, 3.141, 2.718];
        let result = remove_numeric_duplicates(input);
        assert_eq!(result, vec![2.718, 3.141, 3.1415]);
    }
}
use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;

pub fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(file);

    let output_file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    let headers = rdr.headers()?.clone();
    wtr.write_record(&headers)?;

    for result in rdr.records() {
        let record = result?;
        let cleaned_record: Vec<String> = record
            .iter()
            .map(|field| {
                field
                    .trim()
                    .to_uppercase()
                    .replace("\"", "")
                    .replace("\n", " ")
                    .replace("\r", "")
            })
            .collect();
        wtr.write_record(&cleaned_record)?;
    }

    wtr.flush()?;
    println!("Data cleaning completed. Output saved to: {}", output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_clean_csv_data() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "name,age,city\n\"John\",25,\"New York\"\nAlice,30,Boston\n").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        let result = clean_csv_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
        );
        assert!(result.is_ok());

        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(File::open(output_file.path()).unwrap());

        let records: Vec<_> = rdr.records().collect::<Result<_, _>>().unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0][0], "JOHN");
        assert_eq!(records[0][1], "25");
        assert_eq!(records[0][2], "NEW YORK");
    }
}
use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: String) {
        self.records.push(record);
    }

    pub fn deduplicate(&mut self) -> Vec<String> {
        let mut seen = HashSet::new();
        self.records.retain(|record| seen.insert(record.clone()));
        self.records.clone()
    }

    pub fn validate_records(&self) -> Vec<bool> {
        self.records
            .iter()
            .map(|record| !record.trim().is_empty())
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear_all(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test".to_string());
        cleaner.add_record("test".to_string());
        cleaner.add_record("unique".to_string());
        
        let deduped = cleaner.deduplicate();
        assert_eq!(deduped.len(), 2);
        assert!(deduped.contains(&"test".to_string()));
        assert!(deduped.contains(&"unique".to_string()));
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid".to_string());
        cleaner.add_record("   ".to_string());
        cleaner.add_record("".to_string());
        
        let validation_results = cleaner.validate_records();
        assert_eq!(validation_results, vec![true, false, false]);
    }
}