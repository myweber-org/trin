
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn merge_csv_files(
    input_paths: &[&str],
    output_path: &str,
    deduplicate: bool,
    sort_by_column: Option<usize>,
) -> Result<(), Box<dyn Error>> {
    let mut all_records = Vec::new();
    let mut headers = Vec::new();
    let mut header_set = false;

    for input_path in input_paths {
        let path = Path::new(input_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(header_line) = lines.next() {
            let current_headers: Vec<String> = header_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();

            if !header_set {
                headers = current_headers.clone();
                header_set = true;
            } else if headers != current_headers {
                return Err("CSV headers do not match across files".into());
            }

            for line in lines {
                let record = line?;
                if !record.trim().is_empty() {
                    all_records.push(record);
                }
            }
        }
    }

    if deduplicate {
        let unique_records: HashSet<String> = all_records.drain(..).collect();
        all_records = unique_records.into_iter().collect();
    }

    if let Some(col_index) = sort_by_column {
        if col_index < headers.len() {
            all_records.sort_by(|a, b| {
                let a_fields: Vec<&str> = a.split(',').collect();
                let b_fields: Vec<&str> = b.split(',').collect();
                
                if col_index < a_fields.len() && col_index < b_fields.len() {
                    a_fields[col_index].cmp(b_fields[col_index])
                } else {
                    std::cmp::Ordering::Equal
                }
            });
        }
    }

    let mut output_file = File::create(output_path)?;
    writeln!(output_file, "{}", headers.join(","))?;
    
    for record in all_records {
        writeln!(output_file, "{}", record)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_merge_basic() {
        let test_dir = "test_data";
        fs::create_dir_all(test_dir).unwrap();

        let file1 = format!("{}/data1.csv", test_dir);
        let file2 = format!("{}/data2.csv", test_dir);
        let output = format!("{}/merged.csv", test_dir);

        fs::write(&file1, "id,name,value\n1,Alice,100\n2,Bob,200").unwrap();
        fs::write(&file2, "id,name,value\n3,Charlie,300\n4,Diana,400").unwrap();

        let inputs = [file1.as_str(), file2.as_str()];
        let result = merge_csv_files(&inputs, &output, false, None);
        
        assert!(result.is_ok());
        
        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("Alice"));
        assert!(content.contains("Diana"));
        
        fs::remove_dir_all(test_dir).unwrap();
    }
}