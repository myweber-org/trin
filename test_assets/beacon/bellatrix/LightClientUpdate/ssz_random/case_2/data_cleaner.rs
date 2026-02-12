use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;

pub struct DataCleaner {
    input_path: String,
    output_path: String,
}

impl DataCleaner {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        Self {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
        }
    }

    pub fn clean_numeric_columns(&self) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(input_file);
        let output_file = File::create(&self.output_path)?;
        let mut wtr = WriterBuilder::new().from_writer(output_file);

        let headers = rdr.headers()?.clone();
        wtr.write_record(&headers)?;

        for result in rdr.records() {
            let record = result?;
            let mut cleaned_record = Vec::new();

            for field in record.iter() {
                let cleaned_field = field.trim();
                if cleaned_field.is_empty() {
                    cleaned_record.push("0".to_string());
                } else if let Ok(num) = cleaned_field.parse::<f64>() {
                    cleaned_record.push(num.to_string());
                } else {
                    cleaned_record.push(cleaned_field.to_string());
                }
            }

            wtr.write_record(&cleaned_record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn remove_duplicate_rows(&self) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(input_file);
        let output_file = File::create(&self.output_path)?;
        let mut wtr = WriterBuilder::new().from_writer(output_file);

        let headers = rdr.headers()?.clone();
        wtr.write_record(&headers)?;

        let mut seen_records = std::collections::HashSet::new();

        for result in rdr.records() {
            let record = result?;
            let record_str = record.iter().collect::<Vec<&str>>().join(",");
            
            if !seen_records.contains(&record_str) {
                wtr.write_record(&record)?;
                seen_records.insert(record_str);
            }
        }

        wtr.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_clean_numeric_columns() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "id,value,notes").unwrap();
        writeln!(input_file, "1, 42.5 ,test").unwrap();
        writeln!(input_file, "2,,").unwrap();
        writeln!(input_file, "3,invalid,data").unwrap();

        let output_file = NamedTempFile::new().unwrap();
        
        let cleaner = DataCleaner::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        assert!(cleaner.clean_numeric_columns().is_ok());
    }
}