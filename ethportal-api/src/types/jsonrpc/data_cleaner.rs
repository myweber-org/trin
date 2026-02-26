
use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
    deduplicated: bool,
}

impl DataCleaner {
    pub fn new(records: Vec<String>) -> Self {
        DataCleaner {
            records,
            deduplicated: false,
        }
    }

    pub fn deduplicate(&mut self) -> &mut Self {
        if !self.deduplicated {
            let mut seen = HashSet::new();
            self.records = self
                .records
                .iter()
                .filter(|record| seen.insert(record.trim().to_lowercase()))
                .cloned()
                .collect();
            self.deduplicated = true;
        }
        self
    }

    pub fn validate_emails(&self) -> Vec<bool> {
        self.records
            .iter()
            .map(|record| {
                record.contains('@')
                    && record.split('@').count() == 2
                    && !record.starts_with('@')
                    && !record.ends_with('@')
            })
            .collect()
    }

    pub fn filter_valid_emails(&self) -> Vec<String> {
        let validation = self.validate_emails();
        self.records
            .iter()
            .zip(validation.iter())
            .filter_map(|(record, &valid)| if valid { Some(record.clone()) } else { None })
            .collect()
    }

    pub fn get_records(&self) -> &Vec<String> {
        &self.records
    }

    pub fn count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let records = vec![
            "test@example.com".to_string(),
            "TEST@example.com".to_string(),
            "another@test.org".to_string(),
            "test@example.com".to_string(),
        ];

        let mut cleaner = DataCleaner::new(records);
        cleaner.deduplicate();

        assert_eq!(cleaner.count(), 2);
        assert!(cleaner.deduplicated);
    }

    #[test]
    fn test_email_validation() {
        let records = vec![
            "valid@example.com".to_string(),
            "invalid-email".to_string(),
            "@missinglocal.org".to_string(),
            "missingdomain@".to_string(),
        ];

        let cleaner = DataCleaner::new(records);
        let validation = cleaner.validate_emails();

        assert_eq!(validation, vec![true, false, false, false]);
    }

    #[test]
    fn test_filter_valid_emails() {
        let records = vec![
            "user@domain.com".to_string(),
            "not-an-email".to_string(),
            "admin@server.org".to_string(),
        ];

        let cleaner = DataCleaner::new(records);
        let valid_emails = cleaner.filter_valid_emails();

        assert_eq!(valid_emails.len(), 2);
        assert!(valid_emails.contains(&"user@domain.com".to_string()));
        assert!(valid_emails.contains(&"admin@server.org".to_string()));
    }
}
use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(writer);

    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;

    for result in csv_reader.records() {
        let record = result?;
        let cleaned_record: Vec<String> = record
            .iter()
            .map(|field| {
                if field.trim().is_empty() || field.trim().eq_ignore_ascii_case("null") {
                    String::from("")
                } else {
                    field.to_string()
                }
            })
            .collect();

        csv_writer.write_record(&cleaned_record)?;
    }

    csv_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_clean_csv() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "name,age,city").unwrap();
        writeln!(input_file, "John,25,New York").unwrap();
        writeln!(input_file, "Jane,null,London").unwrap();
        writeln!(input_file, "Bob,30,").unwrap();
        writeln!(input_file, "Alice,28,NULL").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        clean_csv(input_file.path().to_str().unwrap(), output_file.path().to_str().unwrap())
            .unwrap();

        let mut rdr = csv::Reader::from_path(output_file.path()).unwrap();
        let records: Vec<_> = rdr.records().collect();
        assert_eq!(records.len(), 4);
        
        let last_record = &records[3];
        assert_eq!(last_record.as_ref().unwrap()[2], "");
    }
}