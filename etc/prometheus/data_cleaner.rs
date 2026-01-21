use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub fn clean_csv_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(output_path)?;

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();

        if !trimmed.is_empty() {
            writeln!(output_file, "{}", trimmed)?;
        }
    }

    Ok(())
}