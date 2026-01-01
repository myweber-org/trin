use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn merge_csv_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
    write_headers: bool,
) -> Result<(), Box<dyn Error>> {
    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    let mut first_file = true;

    for path in input_paths {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let headers = rdr.headers()?.clone();

        if first_file {
            if write_headers {
                writer.write_all(headers.as_bytes())?;
                writer.write_all(b"\n")?;
            }
            first_file = false;
        }

        for result in rdr.records() {
            let record = result?;
            writer.write_all(record.as_slice())?;
            writer.write_all(b"\n")?;
        }
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_csv_files() {
        let csv1 = "id,name\n1,Alice\n2,Bob";
        let csv2 = "id,name\n3,Charlie\n4,Diana";

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        std::fs::write(file1.path(), csv1).unwrap();
        std::fs::write(file2.path(), csv2).unwrap();

        merge_csv_files(
            &[file1.path(), file2.path()],
            output_file.path(),
            true,
        ).unwrap();

        let mut content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        let expected = "id,name\n1,Alice\n2,Bob\n3,Charlie\n4,Diana\n";
        assert_eq!(content, expected);
    }
}