use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    file_path: String,
    delimiter: char,
}

impl CsvProcessor {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        CsvProcessor {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn read_and_filter(&self, column_index: usize, filter_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut filtered_rows = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let columns: Vec<String> = line.split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if line_num == 0 {
                filtered_rows.push(columns.clone());
                continue;
            }

            if let Some(cell_value) = columns.get(column_index) {
                if cell_value == filter_value {
                    filtered_rows.push(columns);
                }
            }
        }

        Ok(filtered_rows)
    }

    pub fn count_rows(&self) -> Result<usize, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines() {
            let line = line?;
            if !line.trim().is_empty() {
                count += 1;
            }
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_filtering() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,status").unwrap();
        writeln!(temp_file, "1,Alice,active").unwrap();
        writeln!(temp_file, "2,Bob,inactive").unwrap();
        writeln!(temp_file, "3,Charlie,active").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let result = processor.read_and_filter(2, "active").unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["id", "name", "status"]);
        assert_eq!(result[1], vec!["1", "Alice", "active"]);
        assert_eq!(result[2], vec!["3", "Charlie", "active"]);
    }

    #[test]
    fn test_row_count() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "header1,header2").unwrap();
        writeln!(temp_file, "value1,value2").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "value3,value4").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let count = processor.count_rows().unwrap();

        assert_eq!(count, 3);
    }
}