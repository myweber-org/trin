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

    pub fn filter_rows<F>(&self, predicate: F) -> Result<Vec<Vec<String>>, Box<dyn Error>>
    where
        F: Fn(&[String]) -> bool,
    {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut filtered_rows = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let columns: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if predicate(&columns) {
                filtered_rows.push(columns);
            }
        }

        Ok(filtered_rows)
    }

    pub fn count_rows(&self) -> Result<usize, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        Ok(reader.lines().count())
    }

    pub fn get_column_names(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let mut reader = BufReader::new(file);
        let mut first_line = String::new();
        reader.read_line(&mut first_line)?;

        Ok(first_line
            .trim()
            .split(self.delimiter)
            .map(|s| s.to_string())
            .collect())
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
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,Paris").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap(), ',');
        
        let adults = processor.filter_rows(|cols| {
            if cols.len() > 1 {
                cols[1].parse::<i32>().unwrap_or(0) >= 30
            } else {
                false
            }
        }).unwrap();

        assert_eq!(adults.len(), 2);
        assert_eq!(adults[0][0], "Alice");
        assert_eq!(adults[1][0], "Charlie");
    }

    #[test]
    fn test_column_names() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id|name|value").unwrap();
        writeln!(temp_file, "1|test|100").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap(), '|');
        let columns = processor.get_column_names().unwrap();
        
        assert_eq!(columns, vec!["id", "name", "value"]);
    }
}