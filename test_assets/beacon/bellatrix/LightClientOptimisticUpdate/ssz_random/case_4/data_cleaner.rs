use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataCleaner {
    input_path: String,
    output_path: String,
    fill_missing: Option<f64>,
}

impl DataCleaner {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        DataCleaner {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            fill_missing: None,
        }
    }

    pub fn set_missing_value_fill(&mut self, value: f64) -> &mut Self {
        self.fill_missing = Some(value);
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
            let mut cleaned_record = Vec::new();

            for field in record.iter() {
                if field.trim().is_empty() {
                    cleaned_record.push(
                        self.fill_missing
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "N/A".to_string()),
                    );
                } else {
                    cleaned_record.push(field.to_string());
                }
            }

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
        writeln!(
            input_file,
            "name,age,salary\nJohn,25,50000\nJane,,60000\nBob,30,"
        )
        .unwrap();

        let output_file = NamedTempFile::new().unwrap();

        let mut cleaner = DataCleaner::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
        );
        cleaner.set_missing_value_fill(0.0);

        assert!(cleaner.clean().is_ok());

        let content = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(content.contains("John,25,50000"));
        assert!(content.contains("Jane,0,60000"));
        assert!(content.contains("Bob,30,0"));
    }
}