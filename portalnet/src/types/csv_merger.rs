use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub fn merge_csv_files(input_paths: &[String], output_path: &str) -> Result<(), Box<dyn Error>> {
    let output_file = File::create(output_path)?;
    let mut writer = io::BufWriter::new(output_file);
    let mut headers_written = false;

    for (index, input_path) in input_paths.iter().enumerate() {
        let path = Path::new(input_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(first_line) = lines.next() {
            let header = first_line?;

            if index == 0 {
                writeln!(writer, "{}", header)?;
                headers_written = true;
            } else if !headers_written {
                writeln!(writer, "{}", header)?;
                headers_written = true;
            }

            for line in lines {
                let line_content = line?;
                if !line_content.trim().is_empty() {
                    writeln!(writer, "{}", line_content)?;
                }
            }
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
        let test_dir = "test_csv_merge";
        fs::create_dir_all(test_dir).unwrap();

        let file1_content = "id,name,value\n1,alpha,100\n2,beta,200";
        let file2_content = "id,name,value\n3,gamma,300\n4,delta,400";

        let file1_path = format!("{}/file1.csv", test_dir);
        let file2_path = format!("{}/file2.csv", test_dir);
        let output_path = format!("{}/merged.csv", test_dir);

        fs::write(&file1_path, file1_content).unwrap();
        fs::write(&file2_path, file2_content).unwrap();

        let input_paths = vec![file1_path.clone(), file2_path.clone()];
        let result = merge_csv_files(&input_paths, &output_path);

        assert!(result.is_ok());

        let merged_content = fs::read_to_string(&output_path).unwrap();
        let expected = "id,name,value\n1,alpha,100\n2,beta,200\n3,gamma,300\n4,delta,400\n";
        assert_eq!(merged_content, expected);

        fs::remove_dir_all(test_dir).unwrap();
    }
}