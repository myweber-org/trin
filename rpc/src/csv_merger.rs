use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn merge_csv_files(
    input_paths: &[impl AsRef<Path>],
    output_path: impl AsRef<Path>,
    has_headers: bool,
) -> Result<(), Box<dyn Error>> {
    let mut output_file = File::create(output_path)?;
    let mut seen_records = HashSet::new();
    let mut first_file = true;

    for input_path in input_paths {
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if has_headers && first_file {
            if let Some(header) = lines.next() {
                writeln!(output_file, "{}", header?)?;
            }
            first_file = false;
            continue;
        }

        if has_headers && !first_file {
            lines.next();
        }

        for line in lines {
            let record = line?;
            if seen_records.insert(record.clone()) {
                writeln!(output_file, "{}", record)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_csv_with_headers() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        std::fs::write(&file1, "id,name\n1,alice\n2,bob\n").unwrap();
        std::fs::write(&file2, "id,name\n2,bob\n3,charlie\n").unwrap();

        merge_csv_files(&[&file1, &file2], &output, true).unwrap();

        let mut content = String::new();
        std::fs::File::open(output)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        assert_eq!(content, "id,name\n1,alice\n2,bob\n3,charlie\n");
    }

    #[test]
    fn test_merge_csv_without_headers() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        std::fs::write(&file1, "1,alice\n2,bob\n").unwrap();
        std::fs::write(&file2, "2,bob\n3,charlie\n").unwrap();

        merge_csv_files(&[&file1, &file2], &output, false).unwrap();

        let mut content = String::new();
        std::fs::File::open(output)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        assert_eq!(content, "1,alice\n2,bob\n3,charlie\n");
    }
}