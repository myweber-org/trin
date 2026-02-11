use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn filter_rows<P: AsRef<Path>>(
        &self,
        file_path: P,
        column_index: usize,
        filter_value: &str,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();
        let mut lines = reader.lines().enumerate();

        if self.has_header {
            lines.next();
        }

        for (line_num, line) in lines {
            let line = line?;
            let columns: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if let Some(cell_value) = columns.get(column_index) {
                if cell_value == filter_value {
                    results.push(columns);
                }
            } else {
                eprintln!("Warning: Line {} has no column at index {}", line_num + 1, column_index);
            }
        }

        Ok(results)
    }

    pub fn count_rows<P: AsRef<Path>>(&self, file_path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let total_lines = reader.lines().count();

        if self.has_header && total_lines > 0 {
            Ok(total_lines - 1)
        } else {
            Ok(total_lines)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_rows() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,London").unwrap();
        writeln!(temp_file, "Bob,25,Paris").unwrap();
        writeln!(temp_file, "Charlie,30,Tokyo").unwrap();

        let processor = CsvProcessor::new(',', true);
        let results = processor.filter_rows(temp_file.path(), 1, "30").unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0][0], "Alice");
        assert_eq!(results[1][0], "Charlie");
    }

    #[test]
    fn test_count_rows() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "header1,header2").unwrap();
        writeln!(temp_file, "data1,data2").unwrap();
        writeln!(temp_file, "data3,data4").unwrap();

        let processor_with_header = CsvProcessor::new(',', true);
        let count_with_header = processor_with_header.count_rows(temp_file.path()).unwrap();
        assert_eq!(count_with_header, 2);

        let processor_no_header = CsvProcessor::new(',', false);
        let count_no_header = processor_no_header.count_rows(temp_file.path()).unwrap();
        assert_eq!(count_no_header, 3);
    }
}