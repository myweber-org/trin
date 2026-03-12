
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct CsvAnalyzer {
    pub file_path: String,
    pub delimiter: char,
    pub has_headers: bool,
}

impl CsvAnalyzer {
    pub fn new(file_path: &str) -> Self {
        CsvAnalyzer {
            file_path: file_path.to_string(),
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
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
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
                
                if numeric_columns[i] && !field.trim().is_empty() {
                    if field.parse::<f64>().is_err() {
                        numeric_columns[i] = false;
                    }
                }
            }
            
            row_count += 1;
        }

        let numeric_column_count = numeric_columns.iter().filter(|&&x| x).count();

        Ok(AnalysisResult {
            row_count,
            column_count,
            empty_cells,
            numeric_column_count,
            file_size: path.metadata()?.len(),
        })
    }
}

pub struct AnalysisResult {
    pub row_count: usize,
    pub column_count: usize,
    pub empty_cells: usize,
    pub numeric_column_count: usize,
    pub file_size: u64,
}

impl AnalysisResult {
    pub fn print_summary(&self) {
        println!("CSV Analysis Summary:");
        println!("  Rows: {}", self.row_count);
        println!("  Columns: {}", self.column_count);
        println!("  Empty cells: {}", self.empty_cells);
        println!("  Numeric columns: {}", self.numeric_column_count);
        println!("  File size: {} bytes", self.file_size);
        println!("  Total cells: {}", self.row_count * self.column_count);
        
        if self.row_count > 0 && self.column_count > 0 {
            let fill_percentage = 100.0 * (1.0 - (self.empty_cells as f64) / (self.row_count * self.column_count) as f64);
            println!("  Data fill: {:.1}%", fill_percentage);
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

        assert_eq!(result.row_count, 3);
        assert_eq!(result.column_count, 3);
        assert_eq!(result.empty_cells, 1);
        assert_eq!(result.numeric_column_count, 2);
    }

    #[test]
    fn test_csv_without_headers() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();

        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap())
            .without_headers();
        let result = analyzer.analyze().unwrap();

        assert_eq!(result.row_count, 2);
        assert_eq!(result.column_count, 3);
    }
}