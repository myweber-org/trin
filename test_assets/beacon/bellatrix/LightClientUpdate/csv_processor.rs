use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvConfig {
    pub delimiter: char,
    pub has_headers: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            has_headers: true,
        }
    }
}

pub fn parse_csv<P: AsRef<Path>>(
    path: P,
    config: &CsvConfig,
) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut lines = reader.lines().enumerate();

    if config.has_headers {
        if let Some((_, first_line)) = lines.next() {
            let headers = first_line?;
            println!("Headers: {}", headers);
        }
    }

    for (line_num, line_result) in lines {
        let line = line_result?;
        let fields: Vec<String> = line
            .split(config.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        if fields.is_empty() {
            return Err(format!("Line {} is empty", line_num + 1).into());
        }

        records.push(fields);
    }

    if records.is_empty() {
        return Err("No data records found".into());
    }

    Ok(records)
}

pub fn validate_records(records: &[Vec<String>]) -> Result<(), Box<dyn Error>> {
    let expected_len = records.first().map(|r| r.len()).unwrap_or(0);

    for (idx, record) in records.iter().enumerate() {
        if record.len() != expected_len {
            return Err(format!(
                "Record {} has {} fields, expected {}",
                idx,
                record.len(),
                expected_len
            )
            .into());
        }

        for (field_idx, field) in record.iter().enumerate() {
            if field.is_empty() {
                return Err(format!(
                    "Empty field at record {}, position {}",
                    idx, field_idx
                )
                .into());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let config = CsvConfig::default();
        let result = parse_csv(temp_file.path(), &config);
        assert!(result.is_ok());

        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_validation_fails_on_missing_fields() {
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string()],
        ];

        let result = validate_records(&records);
        assert!(result.is_err());
    }
}