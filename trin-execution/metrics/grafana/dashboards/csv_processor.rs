use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub fn filter_csv_rows(
    input_path: &str,
    output_path: &str,
    filter_predicate: fn(&[String]) -> bool,
    transform: fn(Vec<String>) -> Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(output_path)?;

    let mut lines = reader.lines();
    if let Some(header) = lines.next() {
        let header = header?;
        writeln!(output_file, "{}", header)?;
    }

    for line in lines {
        let line = line?;
        let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();

        if filter_predicate(&fields) {
            let transformed = transform(fields);
            let output_line = transformed.join(",");
            writeln!(output_file, "{}", output_line)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn test_filter(row: &[String]) -> bool {
        row.get(1).map_or(false, |age| age.parse::<u32>().unwrap_or(0) > 25)
    }

    fn test_transform(mut row: Vec<String>) -> Vec<String> {
        if let Some(name) = row.get_mut(0) {
            *name = name.to_uppercase();
        }
        row
    }

    #[test]
    fn test_csv_processing() {
        let input = "name,age,city\nAlice,30,London\nBob,20,Paris\nCharlie,35,Tokyo";
        let input_path = "test_input.csv";
        let output_path = "test_output.csv";

        fs::write(input_path, input).unwrap();

        filter_csv_rows(input_path, output_path, test_filter, test_transform).unwrap();

        let output = fs::read_to_string(output_path).unwrap();
        let expected = "name,age,city\nALICE,30,LONDON\nCHARLIE,35,TOKYO\n";
        assert_eq!(output, expected);

        fs::remove_file(input_path).unwrap();
        fs::remove_file(output_path).unwrap();
    }
}