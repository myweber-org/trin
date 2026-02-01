use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(output_path)?;

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();

        if !trimmed.is_empty() {
            let cleaned_line: String = trimmed
                .split(',')
                .map(|field| field.trim())
                .collect::<Vec<&str>>()
                .join(",");
            writeln!(output_file, "{}", cleaned_line)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_clean_csv() {
        let input = "test_input.csv";
        let output = "test_output.csv";

        let mut input_file = File::create(input).unwrap();
        writeln!(input_file, "  a , b , c  ").unwrap();
        writeln!(input_file, "").unwrap();
        writeln!(input_file, "x,y,z").unwrap();
        drop(input_file);

        clean_csv(input, output).unwrap();

        let mut output_file = File::open(output).unwrap();
        let mut content = String::new();
        output_file.read_to_string(&mut content).unwrap();

        assert_eq!(content, "a,b,c\nx,y,z\n");

        std::fs::remove_file(input).unwrap();
        std::fs::remove_file(output).unwrap();
    }
}