use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::io;

pub fn clean_csv<R: io::Read, W: io::Write>(input: R, output: W) -> Result<(), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(input);
    let mut wtr = WriterBuilder::new().from_writer(output);

    if let Some(headers) = rdr.headers().ok() {
        wtr.write_record(headers)?;
    }

    for result in rdr.records() {
        let record = result?;
        let cleaned_fields: Vec<String> = record
            .iter()
            .map(|field| field.trim().to_string())
            .filter(|field| !field.is_empty())
            .collect();

        if !cleaned_fields.is_empty() {
            wtr.write_record(&cleaned_fields)?;
        }
    }

    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_clean_csv() {
        let input_data = "name,age,city\nJohn, 25 ,NYC\n,,\n  Alice ,30, Boston \n";
        let expected_output = "name,age,city\nJohn,25,NYC\nAlice,30,Boston\n";

        let input = Cursor::new(input_data);
        let mut output = Cursor::new(Vec::new());

        clean_csv(input, &mut output).unwrap();
        let result = String::from_utf8(output.into_inner()).unwrap();

        assert_eq!(result, expected_output);
    }
}