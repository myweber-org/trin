use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub columns: Vec<String>,
}

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 && self.has_header {
                continue;
            }

            let columns: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !columns.is_empty() && columns.iter().any(|c| !c.is_empty()) {
                records.push(CsvRecord { columns });
            }
        }

        Ok(records)
    }

    pub fn filter_records<F>(&self, records: Vec<CsvRecord>, predicate: F) -> Vec<CsvRecord>
    where
        F: Fn(&CsvRecord) -> bool,
    {
        records.into_iter().filter(predicate).collect()
    }

    pub fn print_records(&self, records: &[CsvRecord]) {
        for (i, record) in records.iter().enumerate() {
            println!("Record {}: {:?}", i + 1, record.columns);
        }
    }
}

pub fn process_csv_sample() -> Result<(), Box<dyn Error>> {
    let processor = CsvProcessor::new(',', true);
    
    let records = processor.parse_file("data/sample.csv")?;
    
    let filtered = processor.filter_records(records, |record| {
        record.columns.len() >= 3 && !record.columns[2].is_empty()
    });
    
    println!("Filtered records count: {}", filtered.len());
    processor.print_records(&filtered);
    
    Ok(())
}