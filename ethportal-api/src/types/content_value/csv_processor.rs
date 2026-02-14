use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(output_path)?;

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        let cleaned_line = clean_csv_line(&line, line_num + 1);
        writeln!(output_file, "{}", cleaned_line)?;
    }

    Ok(())
}

fn clean_csv_line(line: &str, line_num: usize) -> String {
    let mut cleaned_fields = Vec::new();
    let fields: Vec<&str> = line.split(',').collect();

    for (field_num, field) in fields.iter().enumerate() {
        let trimmed = field.trim();
        
        if trimmed.is_empty() {
            cleaned_fields.push("NULL".to_string());
        } else if trimmed.parse::<f64>().is_ok() {
            cleaned_fields.push(trimmed.to_string());
        } else {
            let escaped = trimmed.replace('"', "\"\"");
            cleaned_fields.push(format!("\"{}\"", escaped));
        }
    }

    cleaned_fields.join(",")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_clean_csv_line() {
        assert_eq!(
            clean_csv_line("hello,123,", 1),
            "\"hello\",123,NULL"
        );
        
        assert_eq!(
            clean_csv_line("data with \"quotes\",empty,,456.7", 2),
            "\"data with \"\"quotes\"\"\",\"empty\",NULL,456.7"
        );
    }

    #[test]
    fn test_clean_csv_workflow() -> Result<(), Box<dyn Error>> {
        let test_input = "name,age,score\nJohn,25,95.5\n\"Alice\",,88.0\nBob,30,";
        let input_path = "test_input.csv";
        let output_path = "test_output.csv";

        let mut input_file = File::create(input_path)?;
        write!(input_file, "{}", test_input)?;

        clean_csv(input_path, output_path)?;

        let mut output_file = File::open(output_path)?;
        let mut contents = String::new();
        output_file.read_to_string(&mut contents)?;

        let expected = "\"name\",\"age\",\"score\"\n\"John\",25,95.5\n\"Alice\",NULL,88.0\n\"Bob\",30,NULL\n";
        assert_eq!(contents, expected);

        std::fs::remove_file(input_path)?;
        std::fs::remove_file(output_path)?;

        Ok(())
    }
}