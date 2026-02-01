use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::io;

pub fn clean_csv<R: io::Read, W: io::Write>(input: R, output: W) -> Result<(), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(input);
    let mut wtr = WriterBuilder::new().from_writer(output);

    if let Some(headers) = rdr.headers().ok() {
        wtr.write_record(headers)?;
    }

    for result in rdr.records() {
        let record = result?;
        let cleaned_fields: Vec<String> = record
            .iter()
            .map(|field| field.trim().to_string())
            .filter(|field| !field.is_empty())
            .collect();

        if !cleaned_fields.is_empty() {
            wtr.write_record(&cleaned_fields)?;
        }
    }

    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_clean_csv() {
        let input_data = "name,age,city\nJohn, 25 ,NYC\n,,\n  Alice ,30, Boston \n";
        let expected_output = "name,age,city\nJohn,25,NYC\nAlice,30,Boston\n";

        let input = Cursor::new(input_data);
        let mut output = Cursor::new(Vec::new());

        clean_csv(input, &mut output).unwrap();
        let result = String::from_utf8(output.into_inner()).unwrap();

        assert_eq!(result, expected_output);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataCleaner {
    delimiter: char,
    has_header: bool,
}

impl DataCleaner {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataCleaner {
            delimiter,
            has_header,
        }
    }

    pub fn validate_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<String>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut errors = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line?;

            if self.has_header && line_number == 1 {
                continue;
            }

            let fields: Vec<&str> = line_content.split(self.delimiter).collect();
            
            if fields.len() < 2 {
                errors.push(format!("Line {}: insufficient fields", line_number));
                continue;
            }

            for (i, field) in fields.iter().enumerate() {
                let trimmed = field.trim();
                
                if trimmed.is_empty() {
                    errors.push(format!("Line {}: empty field at column {}", line_number, i + 1));
                }
                
                if trimmed.contains('\n') || trimmed.contains('\r') {
                    errors.push(format!("Line {}: newline character in field at column {}", line_number, i + 1));
                }
            }
        }

        Ok(errors)
    }

    pub fn clean_numeric_field(field: &str) -> Option<f64> {
        let cleaned = field
            .trim()
            .replace(',', "")
            .replace('$', "")
            .replace(' ', "");
        
        cleaned.parse::<f64>().ok()
    }

    pub fn clean_text_field(field: &str) -> String {
        field
            .trim()
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric() && c != ' ' && c != '-', "")
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_numeric_field() {
        assert_eq!(DataCleaner::clean_numeric_field("1,234.56"), Some(1234.56));
        assert_eq!(DataCleaner::clean_numeric_field("$1,234"), Some(1234.0));
        assert_eq!(DataCleaner::clean_numeric_field("invalid"), None);
    }

    #[test]
    fn test_clean_text_field() {
        assert_eq!(DataCleaner::clean_text_field("  HELLO World!  "), "hello world");
        assert_eq!(DataCleaner::clean_text_field("Data-Clean_Test"), "data-clean test");
    }
}