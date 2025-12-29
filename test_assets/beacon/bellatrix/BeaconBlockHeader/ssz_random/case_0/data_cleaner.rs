use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataCleaner {
    input_path: String,
    output_path: String,
    fill_missing: bool,
    missing_value: String,
}

impl DataCleaner {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        DataCleaner {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            fill_missing: false,
            missing_value: "N/A".to_string(),
        }
    }

    pub fn enable_fill_missing(&mut self, value: &str) -> &mut Self {
        self.fill_missing = true;
        self.missing_value = value.to_string();
        self
    }

    pub fn clean(&self) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(Path::new(&self.input_path))?;
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(input_file);

        let output_file = File::create(Path::new(&self.output_path))?;
        let mut writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(output_file);

        let headers = reader.headers()?.clone();
        writer.write_record(&headers)?;

        for result in reader.records() {
            let record = result?;
            let cleaned_record: Vec<String> = record
                .iter()
                .map(|field| {
                    if field.trim().is_empty() && self.fill_missing {
                        self.missing_value.clone()
                    } else {
                        field.to_string()
                    }
                })
                .collect();

            writer.write_record(&cleaned_record)?;
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

    #[test]
    fn test_cleaner_with_missing_values() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "name,age,city").unwrap();
        writeln!(input_file, "Alice,25,New York").unwrap();
        writeln!(input_file, "Bob,,London").unwrap();
        writeln!(input_file, "Charlie,30,").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        let mut cleaner = DataCleaner::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
        );
        cleaner.enable_fill_missing("UNKNOWN");

        let result = cleaner.clean();
        assert!(result.is_ok());
    }
}use std::collections::HashSet;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn normalize_text(&self, input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn deduplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_text(item);
        if self.dedupe_set.contains(&normalized) {
            false
        } else {
            self.dedupe_set.insert(normalized);
            true
        }
    }

    pub fn clean_dataset(&mut self, data: Vec<String>) -> Vec<String> {
        let mut cleaned = Vec::new();
        for item in data {
            if self.deduplicate(&item) {
                cleaned.push(item);
            }
        }
        cleaned
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        let data = vec![
            "Apple".to_string(),
            "apple".to_string(),
            "APPLE".to_string(),
            "Banana".to_string(),
        ];
        
        let cleaned = cleaner.clean_dataset(data);
        assert_eq!(cleaned.len(), 2);
        assert_eq!(cleaner.get_unique_count(), 2);
    }

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_text("  HELLO World  "), "hello world");
    }
}