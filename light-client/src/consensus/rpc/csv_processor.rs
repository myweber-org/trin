
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
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

    pub fn clean_file<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
    ) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(output_path)?;

        let mut cleaned_count = 0;
        let mut lines = reader.lines().enumerate();

        if self.has_header {
            if let Some((_, header_result)) = lines.next() {
                let header = header_result?;
                writeln!(output_file, "{}", header)?;
            }
        }

        for (line_num, line_result) in lines {
            let line = line_result?;
            let cleaned_line = self.clean_line(&line, line_num + 1);

            if let Some(cleaned) = cleaned_line {
                writeln!(output_file, "{}", cleaned)?;
                cleaned_count += 1;
            }
        }

        Ok(cleaned_count)
    }

    fn clean_line(&self, line: &str, line_number: usize) -> Option<String> {
        let fields: Vec<&str> = line.split(self.delimiter).collect();

        if fields.is_empty() {
            eprintln!("Warning: Empty line at line {}", line_number);
            return None;
        }

        let cleaned_fields: Vec<String> = fields
            .iter()
            .map(|field| {
                field
                    .trim()
                    .replace("\"", "")
                    .replace("\'", "")
                    .replace(";", ",")
            })
            .collect();

        let all_fields_valid = cleaned_fields.iter().all(|field| !field.is_empty());

        if all_fields_valid {
            Some(cleaned_fields.join(&self.delimiter.to_string()))
        } else {
            eprintln!("Warning: Invalid data at line {}", line_number);
            None
        }
    }

    pub fn validate_row(&self, row: &str) -> bool {
        let fields: Vec<&str> = row.split(self.delimiter).collect();
        !fields.is_empty() && fields.iter().all(|field| !field.trim().is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processor_validation() {
        let processor = CsvProcessor::new(',', false);
        assert!(processor.validate_row("field1,field2,field3"));
        assert!(!processor.validate_row("field1,,field3"));
        assert!(!processor.validate_row(""));
    }

    #[test]
    fn test_clean_line() {
        let processor = CsvProcessor::new(',', false);
        let test_line = "  data1  , \"data2\" , 'data3' ; test  ";
        let cleaned = processor.clean_line(test_line, 1).unwrap();
        assert_eq!(cleaned, "data1,data2,data3, test");
    }

    #[test]
    fn test_file_processing() -> Result<(), Box<dyn Error>> {
        let input_content = "name,age,city\nJohn,25,\"New York\"\nJane,30,'London'\n,40,Berlin";
        let mut input_file = NamedTempFile::new()?;
        write!(input_file, "{}", input_content)?;

        let output_file = NamedTempFile::new()?;
        let processor = CsvProcessor::new(',', true);

        let cleaned_count = processor.clean_file(input_file.path(), output_file.path())?;
        assert_eq!(cleaned_count, 2);

        let mut output_content = String::new();
        File::open(output_file.path())?.read_to_string(&mut output_content)?;
        assert!(output_content.contains("name,age,city"));
        assert!(output_content.contains("John,25,New York"));
        assert!(output_content.contains("Jane,30,London"));
        assert!(!output_content.contains(",40,Berlin"));

        Ok(())
    }
}