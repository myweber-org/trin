use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub struct LogParser {
    file_path: String,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
        }
    }

    pub fn extract_errors(&self) -> io::Result<Vec<String>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);

        let mut errors = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.to_lowercase().contains("error") {
                errors.push(line);
            }
        }

        Ok(errors)
    }

    pub fn count_errors(&self) -> io::Result<usize> {
        let errors = self.extract_errors()?;
        Ok(errors.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extract_errors() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Starting application").unwrap();
        writeln!(temp_file, "ERROR: Failed to connect").unwrap();
        writeln!(temp_file, "Warning: High memory usage").unwrap();
        writeln!(temp_file, "Another error occurred").unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let errors = parser.extract_errors().unwrap();

        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("ERROR"));
        assert!(errors[1].contains("error"));
    }
}