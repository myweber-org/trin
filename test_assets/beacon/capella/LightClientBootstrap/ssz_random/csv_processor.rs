use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

pub fn filter_and_transform_csv(
    input_path: &str,
    output_path: &str,
    filter_predicate: impl Fn(&csv::StringRecord) -> bool,
    transform_fn: impl Fn(csv::StringRecord) -> csv::StringRecord,
) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new().from_writer(writer);

    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;

    for result in csv_reader.records() {
        let record = result?;
        if filter_predicate(&record) {
            let transformed = transform_fn(record);
            csv_writer.write_record(&transformed)?;
        }
    }

    csv_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv::StringRecord;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_and_transform() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "name,age,city").unwrap();
        writeln!(input_file, "Alice,30,New York").unwrap();
        writeln!(input_file, "Bob,25,London").unwrap();
        writeln!(input_file, "Charlie,35,Tokyo").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        filter_and_transform_csv(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            |record| record.get(1).and_then(|age| age.parse::<u32>().ok()).unwrap_or(0) > 30,
            |mut record| {
                if let Some(city) = record.get(2) {
                    if city == "Tokyo" {
                        record.push_field("JP");
                    } else {
                        record.push_field("Unknown");
                    }
                }
                record
            },
        ).unwrap();

        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(BufReader::new(File::open(output_file.path()).unwrap()));

        let records: Vec<StringRecord> = rdr.records().map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].get(0).unwrap(), "Charlie");
        assert_eq!(records[0].get(3).unwrap(), "JP");
    }
}