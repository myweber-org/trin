use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub fn clean_csv_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(output_path)?;

    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed_line = line.trim();

        if !trimmed_line.is_empty() {
            let cleaned_columns: Vec<String> = trimmed_line
                .split(',')
                .map(|col| col.trim().to_string())
                .collect();

            if cleaned_columns.iter().any(|col| !col.is_empty()) {
                writeln!(output_file, "{}", cleaned_columns.join(","))?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_clean_csv() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";

        let test_data = "  col1, col2 , col3  \n\nvalue1, value2 , value3\n  ,,  \nlast1,last2,last3  ";
        fs::write(test_input, test_data).unwrap();

        clean_csv_file(test_input, test_output).unwrap();

        let result = fs::read_to_string(test_output).unwrap();
        let expected = "col1,col2,col3\nvalue1,value2,value3\nlast1,last2,last3\n";

        assert_eq!(result, expected);

        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}