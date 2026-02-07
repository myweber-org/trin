use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use csv::{ReaderBuilder, WriterBuilder};

pub fn merge_csv_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
) -> Result<(), Box<dyn Error>> {
    if input_paths.is_empty() {
        return Err("No input files provided".into());
    }

    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new().from_writer(&mut writer);

    let mut headers_written = false;

    for input_path in input_paths {
        let file = File::open(input_path)?;
        let mut csv_reader = ReaderBuilder::new().from_reader(file);

        if !headers_written {
            let headers = csv_reader.headers()?.clone();
            csv_writer.write_record(&headers)?;
            headers_written = true;
        }

        for result in csv_reader.records() {
            let record = result?;
            csv_writer.write_record(&record)?;
        }
    }

    csv_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_csv_files() {
        let csv1 = "name,age,city\nAlice,30,New York\nBob,25,London";
        let csv2 = "name,age,city\nCharlie,35,Paris\nDiana,28,Tokyo";

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        std::fs::write(file1.path(), csv1).unwrap();
        std::fs::write(file2.path(), csv2).unwrap();

        merge_csv_files(&[file1.path(), file2.path()], output_file.path()).unwrap();

        let mut content = String::new();
        std::fs::File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        let expected = "name,age,city\nAlice,30,New York\nBob,25,London\nCharlie,35,Paris\nDiana,28,Tokyo\n";
        assert_eq!(content, expected);
    }

    #[test]
    fn test_merge_empty_list() {
        let output_file = NamedTempFile::new().unwrap();
        let result = merge_csv_files::<&Path>(&[], output_file.path());
        assert!(result.is_err());
    }
}