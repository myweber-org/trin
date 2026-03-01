
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub struct LogProcessor {
    log_path: String,
}

impl LogProcessor {
    pub fn new(log_path: &str) -> Self {
        LogProcessor {
            log_path: log_path.to_string(),
        }
    }

    pub fn extract_errors(&self) -> io::Result<Vec<String>> {
        let path = Path::new(&self.log_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut errors = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.contains("ERROR") || line.contains("error") {
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

pub fn process_log_file(path: &str) -> io::Result<Vec<String>> {
    let processor = LogProcessor::new(path);
    processor.extract_errors()
}