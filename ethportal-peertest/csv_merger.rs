use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn merge_csv_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
    write_header: bool,
) -> Result<(), Box<dyn Error>> {
    let mut output_writer = BufWriter::new(File::create(output_path)?);
    let mut first_file = true;

    for input_path in input_paths {
        let mut rdr = csv::Reader::from_path(input_path)?;
        let headers = rdr.headers()?.clone();

        if first_file {
            if write_header {
                writeln!(output_writer, "{}", headers.as_str())?;
            }
            first_file = false;
        }

        for result in rdr.records() {
            let record = result?;
            writeln!(output_writer, "{}", record.as_str())?;
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
        let csv1 = "name,age\nAlice,30\nBob,25";
        let csv2 = "name,age\nCharlie,35\nDiana,28";

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        std::fs::write(file1.path(), csv1).unwrap();
        std::fs::write(file2.path(), csv2).unwrap();

        merge_csv_files(&[file1.path(), file2.path()], output_file.path(), true).unwrap();

        let mut content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        let expected = "name,age\nAlice,30\nBob,25\nCharlie,35\nDiana,28\n";
        assert_eq!(content, expected);
    }
}