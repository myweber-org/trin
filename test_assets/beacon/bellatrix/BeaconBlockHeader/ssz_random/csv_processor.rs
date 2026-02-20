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

    pub fn filter_rows<P, F>(&self, file_path: P, predicate: F) -> Result<Vec<Vec<String>>, Box<dyn Error>>
    where
        P: AsRef<Path>,
        F: Fn(&[String]) -> bool,
    {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut results = Vec::new();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if predicate(&fields) {
                results.push(fields);
            }
        }

        Ok(results)
    }

    pub fn count_matching_rows<P, F>(&self, file_path: P, predicate: F) -> Result<usize, Box<dyn Error>>
    where
        P: AsRef<Path>,
        F: Fn(&[String]) -> bool,
    {
        let matching_rows = self.filter_rows(file_path, predicate)?;
        Ok(matching_rows.len())
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
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,Tokyo").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor
            .filter_rows(temp_file.path(), |fields| fields[1].parse::<i32>().unwrap() > 30)
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0][0], "Charlie");
    }

    #[test]
    fn test_count_matching_rows() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "product,price,stock").unwrap();
        writeln!(temp_file, "Widget,19.99,100").unwrap();
        writeln!(temp_file, "Gadget,29.99,50").unwrap();
        writeln!(temp_file, "Thingy,9.99,200").unwrap();

        let processor = CsvProcessor::new(',', true);
        let count = processor
            .count_matching_rows(temp_file.path(), |fields| {
                fields[2].parse::<i32>().unwrap() > 75
            })
            .unwrap();

        assert_eq!(count, 2);
    }
}