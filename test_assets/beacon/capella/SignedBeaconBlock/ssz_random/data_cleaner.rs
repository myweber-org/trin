use csv::{Reader, Writer};
use std::error::Error;
use std::fs::File;

pub struct DataCleaner {
    fill_missing: String,
}

impl DataCleaner {
    pub fn new(fill_missing: &str) -> Self {
        DataCleaner {
            fill_missing: fill_missing.to_string(),
        }
    }

    pub fn clean_csv(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let mut reader = Reader::from_reader(input_file);
        
        let output_file = File::create(output_path)?;
        let mut writer = Writer::from_writer(output_file);

        let headers = reader.headers()?.clone();
        writer.write_record(&headers)?;

        for result in reader.records() {
            let record = result?;
            let cleaned_record: Vec<String> = record
                .iter()
                .map(|field| {
                    if field.trim().is_empty() {
                        self.fill_missing.clone()
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

    pub fn validate_csv(&self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut reader = Reader::from_reader(file);
        let mut missing_count = 0;

        for result in reader.records() {
            let record = result?;
            for field in record.iter() {
                if field.trim().is_empty() {
                    missing_count += 1;
                }
            }
        }

        Ok(missing_count)
    }
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
        writeln!(input_file, "Jane,,London").unwrap();
        writeln!(input_file, ",30,Paris").unwrap();

        let output_file = NamedTempFile::new().unwrap();
        
        let cleaner = DataCleaner::new("N/A");
        let result = cleaner.clean_csv(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        assert!(result.is_ok());
        
        let missing_count = cleaner.validate_csv(output_file.path().to_str().unwrap()).unwrap();
        assert_eq!(missing_count, 0);
    }
}