use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn merge_csv_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut headers_set = HashSet::new();
    let mut records = Vec::new();
    let mut final_headers = Vec::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(header_line) = lines.next() {
            let header = header_line?;
            let current_headers: Vec<&str> = header.split(',').collect();

            for h in &current_headers {
                headers_set.insert(h.to_string());
            }

            if final_headers.is_empty() {
                final_headers = current_headers.iter().map(|s| s.to_string()).collect();
            }

            for line in lines {
                let record = line?;
                records.push(record);
            }
        }
    }

    if final_headers.is_empty() {
        final_headers = headers_set.into_iter().collect();
        final_headers.sort();
    }

    let mut output_file = File::create(output_path)?;
    writeln!(output_file, "{}", final_headers.join(","))?;

    let mut unique_records = HashSet::new();
    for record in records {
        if unique_records.insert(record.clone()) {
            writeln!(output_file, "{}", record)?;
        }
    }

    output_file.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_merge_csv_files() {
        let test_dir = "test_csv_merge";
        fs::create_dir_all(test_dir).unwrap();

        let file1_content = "id,name,age\n1,Alice,30\n2,Bob,25";
        let file2_content = "id,city,country\n3,London,UK\n4,Paris,FR";
        let file1_path = format!("{}/file1.csv", test_dir);
        let file2_path = format!("{}/file2.csv", test_dir);
        let output_path = format!("{}/merged.csv", test_dir);

        fs::write(&file1_path, file1_content).unwrap();
        fs::write(&file2_path, file2_content).unwrap();

        let inputs = [file1_path.as_str(), file2_path.as_str()];
        let result = merge_csv_files(&inputs, &output_path);

        assert!(result.is_ok());
        let merged_content = fs::read_to_string(&output_path).unwrap();
        assert!(merged_content.contains("id,name,age,city,country"));

        fs::remove_dir_all(test_dir).unwrap();
    }
}