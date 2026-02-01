use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub column_names: Vec<String>,
    pub column_types: HashMap<String, String>,
    pub numeric_columns: Vec<String>,
    pub text_columns: Vec<String>,
}

pub struct CsvAnalyzer {
    path: String,
    delimiter: char,
    has_header: bool,
}

impl CsvAnalyzer {
    pub fn new(path: &str) -> Self {
        CsvAnalyzer {
            path: path.to_string(),
            delimiter: ',',
            has_header: true,
        }
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn without_header(mut self) -> Self {
        self.has_header = false;
        self
    }

    pub fn analyze(&self) -> Result<CsvStats, Box<dyn Error>> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut column_names = Vec::new();
        let mut column_types = HashMap::new();
        let mut numeric_columns = Vec::new();
        let mut text_columns = Vec::new();

        let first_line = lines.next()
            .ok_or("Empty CSV file")??;
        
        let first_fields: Vec<&str> = first_line.split(self.delimiter).collect();
        let column_count = first_fields.len();

        if self.has_header {
            for name in first_fields {
                column_names.push(name.trim().to_string());
                column_types.insert(name.trim().to_string(), "unknown".to_string());
            }
        } else {
            for i in 0..column_count {
                column_names.push(format!("Column_{}", i + 1));
                column_types.insert(format!("Column_{}", i + 1), "unknown".to_string());
            }
            lines = BufReader::new(File::open(&self.path)?).lines();
        }

        let mut row_count = 0;
        for line_result in lines {
            let line = line_result?;
            if line.trim().is_empty() {
                continue;
            }
            
            let fields: Vec<&str> = line.split(self.delimiter).collect();
            if fields.len() != column_count {
                continue;
            }

            for (i, field) in fields.iter().enumerate() {
                let col_name = &column_names[i];
                let current_type = column_types.get(col_name).unwrap();

                if current_type == "unknown" {
                    if field.trim().parse::<f64>().is_ok() {
                        column_types.insert(col_name.clone(), "numeric".to_string());
                    } else {
                        column_types.insert(col_name.clone(), "text".to_string());
                    }
                } else if current_type == "numeric" && field.trim().parse::<f64>().is_err() {
                    column_types.insert(col_name.clone(), "text".to_string());
                }
            }
            
            row_count += 1;
        }

        for (col_name, col_type) in &column_types {
            match col_type.as_str() {
                "numeric" => numeric_columns.push(col_name.clone()),
                "text" => text_columns.push(col_name.clone()),
                _ => {}
            }
        }

        Ok(CsvStats {
            row_count,
            column_count,
            column_names,
            column_types,
            numeric_columns,
            text_columns,
        })
    }

    pub fn filter_rows<F>(&self, predicate: F) -> Result<Vec<Vec<String>>, Box<dyn Error>>
    where
        F: Fn(&[String]) -> bool,
    {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut result = Vec::new();
        let mut skip_header = self.has_header;

        for line_result in lines {
            let line = line_result?;
            if skip_header {
                skip_header = false;
                continue;
            }
            
            if line.trim().is_empty() {
                continue;
            }
            
            let fields: Vec<String> = line.split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if predicate(&fields) {
                result.push(fields);
            }
        }

        Ok(result)
    }
}

pub fn summarize_csv(path: &str) -> Result<(), Box<dyn Error>> {
    let analyzer = CsvAnalyzer::new(path);
    let stats = analyzer.analyze()?;

    println!("CSV Analysis Summary:");
    println!("File: {}", path);
    println!("Total Rows: {}", stats.row_count);
    println!("Total Columns: {}", stats.column_count);
    println!("Column Names: {:?}", stats.column_names);
    println!("Numeric Columns: {:?}", stats.numeric_columns);
    println!("Text Columns: {:?}", stats.text_columns);

    Ok(())
}