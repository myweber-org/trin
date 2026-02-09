use csv::{Reader, Writer};
use std::error::Error;
use std::fs::File;

pub fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    let headers = reader.headers()?.clone();
    writer.write_record(&headers)?;

    for result in reader.records() {
        let record = result?;
        let cleaned_record: Vec<String> = record
            .iter()
            .map(|field| {
                field
                    .trim()
                    .to_lowercase()
                    .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
            })
            .collect();
        writer.write_record(&cleaned_record)?;
    }

    writer.flush()?;
    Ok(())
}