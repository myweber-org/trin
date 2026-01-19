
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

            match cleaned_line {
                Ok(cleaned) => {
                    writeln!(output_file, "{}", cleaned)?;
                    cleaned_count += 1;
                }
                Err(e) => {
                    eprintln!("Warning at line {}: {}", line_num + 1, e);
                }
            }
        }

        Ok(cleaned_count)
    }

    fn clean_line(&self, line: &str, line_number: usize) -> Result<String, String> {
        let fields: Vec<&str> = line.split(self.delimiter).collect();

        if fields.is_empty() {
            return Err(format!("Empty line at {}", line_number));
        }

        let cleaned_fields: Vec<String> = fields
            .iter()
            .enumerate()
            .map(|(i, &field)| {
                let trimmed = field.trim();
                if trimmed.is_empty() {
                    format!("MISSING_{}", i)
                } else {
                    trimmed.to_string()
                }
            })
            .collect();

        Ok(cleaned_fields.join(&self.delimiter.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_clean_line() {
        let processor = CsvProcessor::new(',', false);
        let result = processor.clean_line("a,b,c", 1);
        assert_eq!(result.unwrap(), "a,b,c");
    }

    #[test]
    fn test_clean_line_with_spaces() {
        let processor = CsvProcessor::new(',', false);
        let result = processor.clean_line("  a  , b ,  c  ", 1);
        assert_eq!(result.unwrap(), "a,b,c");
    }

    #[test]
    fn test_clean_line_empty_fields() {
        let processor = CsvProcessor::new(',', false);
        let result = processor.clean_line("a,,c", 1);
        assert_eq!(result.unwrap(), "a,MISSING_1,c");
    }

    #[test]
    fn test_clean_file() {
        let input_content = "name,age,city\nJohn,25,NYC\nJane,,London\n,30,Paris";
        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "{}", input_content).unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let processor = CsvProcessor::new(',', true);

        let result = processor.clean_file(input_file.path(), output_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);

        let mut output_content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();

        let expected = "name,age,city\nJohn,25,NYC\nJane,MISSING_1,London\nMISSING_0,30,Paris\n";
        assert_eq!(output_content, expected);
    }
}