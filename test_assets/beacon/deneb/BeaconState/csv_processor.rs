use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: usize,
    filter_value: String,
}

impl CsvProcessor {
    pub fn new(input_path: &str, output_path: &str, filter_column: usize, filter_value: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            filter_column,
            filter_value: filter_value.to_string(),
        }
    }

    pub fn process(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        let mut processed_count = 0;

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            let columns: Vec<&str> = line.split(',').collect();

            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if columns.get(self.filter_column).map_or(false, |&val| val == self.filter_value) {
                let transformed_line = self.transform_record(&columns);
                writeln!(output_file, "{}", transformed_line)?;
                processed_count += 1;
            }
        }

        Ok(processed_count)
    }

    fn transform_record(&self, columns: &[&str]) -> String {
        let mut transformed: Vec<String> = columns.iter().map(|&s| s.to_string()).collect();
        if transformed.len() > 2 {
            transformed.swap(1, 2);
        }
        transformed.join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_csv_processing() {
        let test_input = "id,name,department,salary\n1,Alice,Engineering,75000\n2,Bob,Marketing,65000\n3,Charlie,Engineering,80000";
        let input_path = "test_input.csv";
        let output_path = "test_output.csv";

        std::fs::write(input_path, test_input).unwrap();
        
        let processor = CsvProcessor::new(input_path, output_path, 2, "Engineering");
        let result = processor.process();
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);

        let mut output_content = String::new();
        File::open(output_path)
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();

        let expected = "id,name,department,salary\n1,Engineering,Alice,75000\n3,Engineering,Charlie,80000\n";
        assert_eq!(output_content, expected);

        std::fs::remove_file(input_path).unwrap();
        std::fs::remove_file(output_path).unwrap();
    }
}