
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataCleaner {
    delimiter: char,
    has_headers: bool,
}

impl DataCleaner {
    pub fn new(delimiter: char, has_headers: bool) -> Self {
        DataCleaner {
            delimiter,
            has_headers,
        }
    }

    pub fn validate_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(self.delimiter as u8)
            .has_headers(self.has_headers)
            .from_reader(file);

        let mut record_count = 0;
        for result in rdr.records() {
            let record = result?;
            if record.is_empty() {
                return Err("Empty record found".into());
            }
            record_count += 1;
        }

        Ok(record_count)
    }

    pub fn filter_records<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
        predicate: impl Fn(&csv::StringRecord) -> bool,
    ) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let output_file = File::create(output_path)?;

        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(self.delimiter as u8)
            .has_headers(self.has_headers)
            .from_reader(input_file);

        let mut wtr = csv::WriterBuilder::new()
            .delimiter(self.delimiter as u8)
            .from_writer(output_file);

        if self.has_headers {
            let headers = rdr.headers()?;
            wtr.write_record(headers)?;
        }

        let mut filtered_count = 0;
        for result in rdr.records() {
            let record = result?;
            if predicate(&record) {
                wtr.write_record(&record)?;
                filtered_count += 1;
            }
        }

        wtr.flush()?;
        Ok(filtered_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_validate_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let cleaner = DataCleaner::new(',', true);
        let result = cleaner.validate_csv(temp_file.path());
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_filter_records() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "name,age,city").unwrap();
        writeln!(input_file, "Alice,30,New York").unwrap();
        writeln!(input_file, "Bob,25,London").unwrap();
        writeln!(input_file, "Charlie,35,Paris").unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let cleaner = DataCleaner::new(',', true);

        let filtered = cleaner
            .filter_records(
                input_file.path(),
                output_file.path(),
                |record| record.get(1).and_then(|age| age.parse::<i32>().ok()) > Some(30),
            )
            .unwrap();

        assert_eq!(filtered, 1);
    }
}