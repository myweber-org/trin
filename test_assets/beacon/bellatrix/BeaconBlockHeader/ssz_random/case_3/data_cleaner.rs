use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut seen_lines = HashSet::new();
    let mut cleaned_data = Vec::new();

    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed_line = line.trim();

        if !trimmed_line.is_empty() && seen_lines.insert(trimmed_line.to_string()) {
            cleaned_data.push(trimmed_line.to_string());
        }
    }

    let mut output_file = File::create(output_path)?;
    for cleaned_line in cleaned_data {
        writeln!(output_file, "{}", cleaned_line)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_clean_csv() -> Result<(), Box<dyn Error>> {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";

        let mut input_file = File::create(test_input)?;
        writeln!(input_file, "  apple  ")?;
        writeln!(input_file, "banana")?;
        writeln!(input_file, "  apple  ")?;
        writeln!(input_file, "")?;
        writeln!(input_file, "cherry")?;
        drop(input_file);

        clean_csv(test_input, test_output)?;

        let mut output_file = File::open(test_output)?;
        let mut content = String::new();
        output_file.read_to_string(&mut content)?;

        let expected = "apple\nbanana\ncherry\n";
        assert_eq!(content, expected);

        std::fs::remove_file(test_input)?;
        std::fs::remove_file(test_output)?;

        Ok(())
    }
}