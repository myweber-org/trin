use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvProcessor {
    delimiter: char,
    has_headers: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_headers: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_headers,
        }
    }

    pub fn filter_columns<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
        selected_columns: &[usize],
    ) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(output_path)?;

        let mut lines = reader.lines();
        
        if self.has_headers {
            if let Some(header_line) = lines.next() {
                let headers: Vec<String> = header_line?
                    .split(self.delimiter)
                    .map(String::from)
                    .collect();
                
                let filtered_headers: Vec<String> = selected_columns
                    .iter()
                    .filter_map(|&idx| headers.get(idx).cloned())
                    .collect();
                
                writeln!(output_file, "{}", filtered_headers.join(&self.delimiter.to_string()))?;
            }
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<&str> = line.split(self.delimiter).collect();
            
            let filtered_fields: Vec<String> = selected_columns
                .iter()
                .filter_map(|&idx| fields.get(idx).map(|&s| s.to_string()))
                .collect();
            
            writeln!(output_file, "{}", filtered_fields.join(&self.delimiter.to_string()))?;
        }

        Ok(())
    }

    pub fn count_rows<P: AsRef<Path>>(&self, file_path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line_result in reader.lines() {
            let line = line_result?;
            if !line.trim().is_empty() {
                count += 1;
            }
        }

        if self.has_headers && count > 0 {
            count -= 1;
        }

        Ok(count)
    }
}

pub fn read_csv_preview<P: AsRef<Path>>(
    file_path: P,
    delimiter: char,
    preview_lines: usize,
) -> Result<Vec<String>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut preview = Vec::new();

    for (i, line_result) in reader.lines().enumerate() {
        if i >= preview_lines {
            break;
        }
        preview.push(line_result?);
    }

    Ok(preview)
}