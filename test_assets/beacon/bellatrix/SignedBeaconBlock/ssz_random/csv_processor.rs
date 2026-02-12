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
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let columns: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !columns.is_empty() {
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

pub fn process_csv_sample() -> io::Result<()> {
    let processor = CsvProcessor::new(',', true);
    
    match processor.parse_file("data/sample.csv") {
        Ok(records) => {
            println!("Found {} records", records.len());
            
            let filtered = processor.filter_records(records, |rec| {
                rec.columns.len() >= 3 && !rec.columns[0].is_empty()
            });
            
            println!("Filtered to {} records", filtered.len());
            processor.print_records(&filtered);
        }
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }
    
    Ok(())
}