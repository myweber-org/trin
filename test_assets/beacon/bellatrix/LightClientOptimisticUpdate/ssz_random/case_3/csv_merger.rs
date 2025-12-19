use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use csv::{ReaderBuilder, WriterBuilder};

pub fn merge_csv_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
    has_headers: bool,
) -> Result<(), Box<dyn Error>> {
    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new().from_writer(&mut writer);

    let mut headers_written = false;

    for (index, input_path) in input_paths.iter().enumerate() {
        let file = File::open(input_path)?;
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(has_headers)
            .from_reader(file);

        let headers = csv_reader.headers()?.clone();

        if index == 0 && has_headers {
            csv_writer.write_record(&headers)?;
            headers_written = true;
        }

        for result in csv_reader.records() {
            let record = result?;
            csv_writer.write_record(&record)?;
        }

        if !has_headers && index == 0 && !headers_written {
            csv_writer.write_record(&headers)?;
            headers_written = true;
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
    fn test_merge_csv_with_headers() {
        let csv1 = "name,age\nAlice,30\nBob,25";
        let csv2 = "name,age\nCharlie,35\nDiana,28";

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        std::fs::write(file1.path(), csv1).unwrap();
        std::fs::write(file2.path(), csv2).unwrap();

        let inputs = [file1.path(), file2.path()];
        merge_csv_files(&inputs, output_file.path(), true).unwrap();

        let mut content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        let expected = "name,age\nAlice,30\nBob,25\nCharlie,35\nDiana,28\n";
        assert_eq!(content, expected);
    }

    #[test]
    fn test_merge_csv_without_headers() {
        let csv1 = "Alice,30\nBob,25";
        let csv2 = "Charlie,35\nDiana,28";

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        std::fs::write(file1.path(), csv1).unwrap();
        std::fs::write(file2.path(), csv2).unwrap();

        let inputs = [file1.path(), file2.path()];
        merge_csv_files(&inputs, output_file.path(), false).unwrap();

        let mut content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        let expected = "Alice,30\nBob,25\nCharlie,35\nDiana,28\n";
        assert_eq!(content, expected);
    }
}