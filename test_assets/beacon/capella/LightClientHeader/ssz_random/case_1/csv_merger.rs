use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn merge_csv_files(
    input_paths: &[&str],
    output_path: &str,
    deduplicate: bool,
) -> Result<(), Box<dyn Error>> {
    let mut output_file = File::create(output_path)?;
    let mut header_written = false;
    let mut seen_rows = HashSet::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", input_path);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(first_line) = lines.next() {
            let header = first_line?;

            if !header_written {
                writeln!(output_file, "{}", header)?;
                header_written = true;
            }

            for line in lines {
                let row = line?;
                if deduplicate {
                    if seen_rows.insert(row.clone()) {
                        writeln!(output_file, "{}", row)?;
                    }
                } else {
                    writeln!(output_file, "{}", row)?;
                }
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
    fn test_merge_without_deduplication() {
        let input1 = "test1.csv";
        let input2 = "test2.csv";
        let output = "merged_output.csv";

        fs::write(input1, "id,name\n1,Alice\n2,Bob").unwrap();
        fs::write(input2, "id,name\n3,Charlie\n4,Diana").unwrap();

        let result = merge_csv_files(&[input1, input2], output, false);
        assert!(result.is_ok());

        let content = fs::read_to_string(output).unwrap();
        assert!(content.contains("Alice"));
        assert!(content.contains("Diana"));

        fs::remove_file(input1).unwrap();
        fs::remove_file(input2).unwrap();
        fs::remove_file(output).unwrap();
    }

    #[test]
    fn test_merge_with_deduplication() {
        let input1 = "dup1.csv";
        let input2 = "dup2.csv";
        let output = "dedup_output.csv";

        fs::write(input1, "id,name\n1,Alice\n2,Bob").unwrap();
        fs::write(input2, "id,name\n2,Bob\n3,Charlie").unwrap();

        let result = merge_csv_files(&[input1, input2], output, true);
        assert!(result.is_ok());

        let content = fs::read_to_string(output).unwrap();
        let bob_count = content.matches("Bob").count();
        assert_eq!(bob_count, 1);

        fs::remove_file(input1).unwrap();
        fs::remove_file(input2).unwrap();
        fs::remove_file(output).unwrap();
    }
}