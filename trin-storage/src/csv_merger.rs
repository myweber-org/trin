use std::error::Error;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

pub fn merge_csv_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
    write_headers: bool,
) -> Result<(), Box<dyn Error>> {
    let mut output_writer = BufWriter::new(File::create(output_path)?);
    let mut first_file = true;

    for input_path in input_paths {
        let mut rdr = csv::Reader::from_path(input_path)?;
        let headers = rdr.headers()?.clone();

        if first_file {
            if write_headers {
                output_writer.write_all(headers.as_bytes())?;
                output_writer.write_all(b"\n")?;
            }
            first_file = false;
        } else if write_headers {
            let current_headers = rdr.headers()?;
            if headers != current_headers {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "CSV files have different headers",
                )
                .into());
            }
        }

        for result in rdr.records() {
            let record = result?;
            output_writer.write_all(record.as_slice().as_bytes())?;
            output_writer.write_all(b"\n")?;
        }
    }

    output_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_csv_files() {
        let file1_content = "id,name\n1,Alice\n2,Bob\n";
        let file2_content = "id,name\n3,Charlie\n4,David\n";

        let mut temp_file1 = NamedTempFile::new().unwrap();
        let mut temp_file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        temp_file1.write_all(file1_content.as_bytes()).unwrap();
        temp_file2.write_all(file2_content.as_bytes()).unwrap();

        let input_paths = [temp_file1.path(), temp_file2.path()];

        merge_csv_files(&input_paths, output_file.path(), true).unwrap();

        let mut merged_content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut merged_content)
            .unwrap();

        let expected = "id,name\n1,Alice\n2,Bob\n3,Charlie\n4,David\n";
        assert_eq!(merged_content, expected);
    }
}