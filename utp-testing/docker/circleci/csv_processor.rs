use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

pub fn filter_and_transform_csv(
    input_path: &str,
    output_path: &str,
    filter_predicate: fn(&csv::StringRecord) -> bool,
    transform_fn: fn(csv::StringRecord) -> csv::StringRecord,
) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(writer);

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
    use tempfile::NamedTempFile;

    fn sample_filter(record: &StringRecord) -> bool {
        record.get(1).map(|s| s == "active").unwrap_or(false)
    }

    fn sample_transform(mut record: StringRecord) -> StringRecord {
        if let Some(status) = record.get_mut(1) {
            *status = status.to_uppercase();
        }
        record
    }

    #[test]
    fn test_filter_and_transform() {
        let input_data = "id,status,value\n1,active,100\n2,inactive,200\n3,active,300";
        let mut input_file = NamedTempFile::new().unwrap();
        std::fs::write(input_file.path(), input_data).unwrap();

        let output_file = NamedTempFile::new().unwrap();

        filter_and_transform_csv(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            sample_filter,
            sample_transform,
        ).unwrap();

        let output = std::fs::read_to_string(output_file.path()).unwrap();
        assert_eq!(output, "id,status,value\n1,ACTIVE,100\n3,ACTIVE,300\n");
    }
}