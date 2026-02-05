use csv::{Reader, Writer};
use std::error::Error;
use std::fs::File;

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    let headers = reader.headers()?.clone();
    writer.write_record(&headers)?;

    for result in reader.records() {
        let record = result?;
        let cleaned_record: Vec<String> = record
            .iter()
            .map(|field| field.trim().to_string())
            .collect();
        writer.write_record(&cleaned_record)?;
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_clean_csv() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "name , age, city  ").unwrap();
        writeln!(input_file, "Alice, 25 ,  New York").unwrap();
        writeln!(input_file, "Bob  ,30,London ").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        clean_csv(input_file.path().to_str().unwrap(), output_file.path().to_str().unwrap()).unwrap();

        let mut reader = Reader::from_path(output_file.path()).unwrap();
        let records: Vec<_> = reader.records().collect::<Result<_, _>>().unwrap();

        assert_eq!(records[0][0], "name");
        assert_eq!(records[0][1], "age");
        assert_eq!(records[0][2], "city");
        assert_eq!(records[1][0], "Alice");
        assert_eq!(records[1][1], "25");
        assert_eq!(records[1][2], "New York");
        assert_eq!(records[2][0], "Bob");
        assert_eq!(records[2][1], "30");
        assert_eq!(records[2][2], "London");
    }
}