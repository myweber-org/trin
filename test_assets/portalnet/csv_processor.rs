use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: usize,
    filter_value: String,
    transform_column: usize,
    transform_fn: fn(&str) -> String,
}

impl CsvProcessor {
    pub fn new(
        input_path: String,
        output_path: String,
        filter_column: usize,
        filter_value: String,
        transform_column: usize,
        transform_fn: fn(&str) -> String,
    ) -> Self {
        CsvProcessor {
            input_path,
            output_path,
            filter_column,
            filter_value,
            transform_column,
            transform_fn,
        }
    }

    pub fn process(&self) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = io::BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();

            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if parts.get(self.filter_column).map_or(false, |&val| val == self.filter_value) {
                let mut transformed_parts = parts.clone();
                if let Some(cell) = transformed_parts.get_mut(self.transform_column) {
                    *cell = &(self.transform_fn)(cell);
                }
                writeln!(output_file, "{}", transformed_parts.join(","))?;
            }
        }

        Ok(())
    }
}

fn uppercase_transform(value: &str) -> String {
    value.to_uppercase()
}

fn numeric_double_transform(value: &str) -> String {
    value.parse::<f64>().map_or(value.to_string(), |num| (num * 2.0).to_string())
}

pub fn process_csv_files() -> Result<(), Box<dyn Error>> {
    let processor1 = CsvProcessor::new(
        "input_data.csv".to_string(),
        "filtered_data.csv".to_string(),
        2,
        "active".to_string(),
        3,
        uppercase_transform,
    );

    let processor2 = CsvProcessor::new(
        "sales.csv".to_string(),
        "adjusted_sales.csv".to_string(),
        1,
        "2024".to_string(),
        4,
        numeric_double_transform,
    );

    processor1.process()?;
    processor2.process()?;

    println!("CSV processing completed successfully");
    Ok(())
}