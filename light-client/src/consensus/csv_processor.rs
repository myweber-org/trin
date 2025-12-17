
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: Option<usize>,
    filter_value: Option<String>,
    transform_column: Option<usize>,
    transform_fn: Option<Box<dyn Fn(&str) -> String>>,
}

impl CsvProcessor {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            filter_column: None,
            filter_value: None,
            transform_column: None,
            transform_fn: None,
        }
    }

    pub fn set_filter(mut self, column: usize, value: &str) -> Self {
        self.filter_column = Some(column);
        self.filter_value = Some(value.to_string());
        self
    }

    pub fn set_transform<F>(mut self, column: usize, transform_fn: F) -> Self
    where
        F: Fn(&str) -> String + 'static,
    {
        self.transform_column = Some(column);
        self.transform_fn = Some(Box::new(transform_fn));
        self
    }

    pub fn process(&self) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(Path::new(&self.input_path))?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(Path::new(&self.output_path))?;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let mut columns: Vec<String> = line.split(',').map(|s| s.to_string()).collect();

            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if let (Some(filter_col), Some(filter_val)) = (&self.filter_column, &self.filter_value) {
                if columns.get(*filter_col).map(|s| s.as_str()) != Some(filter_val.as_str()) {
                    continue;
                }
            }

            if let (Some(transform_col), Some(transform_fn)) = (&self.transform_column, &self.transform_fn) {
                if let Some(cell) = columns.get_mut(*transform_col) {
                    *cell = transform_fn(cell);
                }
            }

            writeln!(output_file, "{}", columns.join(","))?;
        }

        Ok(())
    }
}

pub fn uppercase_transform(value: &str) -> String {
    value.to_uppercase()
}

pub fn reverse_transform(value: &str) -> String {
    value.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_csv_processing() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";

        let content = "id,name,value\n1,apple,10\n2,banana,20\n3,apple,30\n";
        fs::write(test_input, content).unwrap();

        let processor = CsvProcessor::new(test_input, test_output)
            .set_filter(1, "apple")
            .set_transform(2, |v| format!("val_{}", v));

        assert!(processor.process().is_ok());

        let output = fs::read_to_string(test_output).unwrap();
        assert!(output.contains("1,apple,val_10"));
        assert!(output.contains("3,apple,val_30"));
        assert!(!output.contains("banana"));

        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}