
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
}use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn merge_csv_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn Error>> {
    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    let mut headers_written = false;

    for (index, input_path) in input_paths.iter().enumerate() {
        let path = Path::new(input_path);
        let mut rdr = csv::Reader::from_path(path)?;
        let headers = rdr.headers()?.clone();

        if index == 0 {
            writer.write_all(headers.as_bytes())?;
            writer.write_all(b"\n")?;
            headers_written = true;
        } else if headers != rdr.headers()? {
            eprintln!("Warning: Headers in {} differ from first file. Skipping header.", input_path);
        }

        for result in rdr.records() {
            let record = result?;
            writer.write_all(record.as_slice().as_bytes())?;
            writer.write_all(b"\n")?;
        }
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_merge_csv_files() {
        let data1 = "name,age\nAlice,30\nBob,25";
        let data2 = "name,age\nCharlie,35\nDiana,28";
        fs::write("test1.csv", data1).unwrap();
        fs::write("test2.csv", data2).unwrap();

        let inputs = vec!["test1.csv", "test2.csv"];
        merge_csv_files(&inputs, "merged.csv").unwrap();

        let merged = fs::read_to_string("merged.csv").unwrap();
        let expected = "name,age\nAlice,30\nBob,25\nCharlie,35\nDiana,28\n";
        assert_eq!(merged, expected);

        fs::remove_file("test1.csv").unwrap();
        fs::remove_file("test2.csv").unwrap();
        fs::remove_file("merged.csv").unwrap();
    }
}