
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct CsvAnalyzer {
    path: String,
    delimiter: char,
    has_headers: bool,
}

impl CsvAnalyzer {
    pub fn new(path: &str) -> Self {
        CsvAnalyzer {
            path: path.to_string(),
            delimiter: ',',
            has_headers: true,
        }
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn without_headers(mut self) -> Self {
        self.has_headers = false;
        self
    }

    pub fn analyze(&self) -> Result<AnalysisResult, Box<dyn Error>> {
        let file = File::open(Path::new(&self.path))?;
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(self.delimiter as u8)
            .has_headers(self.has_headers)
            .from_reader(file);

        let mut row_count = 0;
        let mut column_count = 0;
        let mut empty_cells = 0;
        let mut numeric_columns = Vec::new();

        for result in rdr.records() {
            let record = result?;
            
            if row_count == 0 {
                column_count = record.len();
                numeric_columns = vec![true; column_count];
            }

            for (i, field) in record.iter().enumerate() {
                if field.trim().is_empty() {
                    empty_cells += 1;
                }
                
                if numeric_columns[i] && field.parse::<f64>().is_err() {
                    numeric_columns[i] = false;
                }
            }
            
            row_count += 1;
        }

        let numeric_column_count = numeric_columns.iter().filter(|&&x| x).count();

        Ok(AnalysisResult {
            file_path: self.path.clone(),
            total_rows: row_count,
            total_columns: column_count,
            empty_cells,
            numeric_columns: numeric_column_count,
            delimiter: self.delimiter,
        })
    }
}

pub struct AnalysisResult {
    pub file_path: String,
    pub total_rows: usize,
    pub total_columns: usize,
    pub empty_cells: usize,
    pub numeric_columns: usize,
    pub delimiter: char,
}

impl AnalysisResult {
    pub fn print_summary(&self) {
        println!("CSV Analysis Summary");
        println!("====================");
        println!("File: {}", self.file_path);
        println!("Rows: {}", self.total_rows);
        println!("Columns: {}", self.total_columns);
        println!("Empty cells: {}", self.empty_cells);
        println!("Numeric columns: {}", self.numeric_columns);
        println!("Delimiter: '{}'", self.delimiter);
        
        let total_cells = self.total_rows * self.total_columns;
        if total_cells > 0 {
            let completeness = 100.0 * (1.0 - (self.empty_cells as f64 / total_cells as f64));
            println!("Data completeness: {:.1}%", completeness);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_csv_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,").unwrap();

        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap());
        let result = analyzer.analyze().unwrap();

        assert_eq!(result.total_rows, 3);
        assert_eq!(result.total_columns, 3);
        assert_eq!(result.empty_cells, 1);
        assert_eq!(result.numeric_columns, 2);
    }

    #[test]
    fn test_csv_without_headers() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();

        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap())
            .without_headers();
        let result = analyzer.analyze().unwrap();

        assert_eq!(result.total_rows, 2);
        assert_eq!(result.total_columns, 3);
    }
}