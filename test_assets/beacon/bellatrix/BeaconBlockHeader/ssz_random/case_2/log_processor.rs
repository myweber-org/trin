use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub struct LogProcessor;

impl LogProcessor {
    pub fn extract_errors<P: AsRef<Path>>(log_path: P) -> io::Result<Vec<String>> {
        let file = File::open(log_path)?;
        let reader = BufReader::new(file);
        let mut errors = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.to_lowercase().contains("error") {
                errors.push(line);
            }
        }

        Ok(errors)
    }

    pub fn count_errors<P: AsRef<Path>>(log_path: P) -> io::Result<usize> {
        let errors = Self::extract_errors(log_path)?;
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
        writeln!(temp_file, "INFO: Application started").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "WARN: High memory usage").unwrap();
        writeln!(temp_file, "ERROR: File not found").unwrap();

        let errors = LogProcessor::extract_errors(temp_file.path()).unwrap();
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("Database connection failed"));
        assert!(errors[1].contains("File not found"));
    }

    #[test]
    fn test_count_errors() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: System boot").unwrap();
        writeln!(temp_file, "ERROR: Network timeout").unwrap();
        writeln!(temp_file, "ERROR: Permission denied").unwrap();

        let count = LogProcessor::count_errors(temp_file.path()).unwrap();
        assert_eq!(count, 2);
    }
}