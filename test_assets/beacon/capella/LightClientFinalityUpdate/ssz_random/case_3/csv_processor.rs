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
            
            if !columns.is_empty() && !columns.iter().all(|c| c.is_empty()) {
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
    
    let sample_data = "id,name,value\n1,item_a,100\n2,item_b,200\n3,item_c,150";
    let temp_path = "sample_data.csv";
    
    std::fs::write(temp_path, sample_data)?;
    
    let records = processor.parse_file(temp_path)?;
    
    println!("Total records: {}", records.len());
    
    let filtered = processor.filter_records(records, |record| {
        record.columns.get(2)
            .and_then(|v| v.parse::<i32>().ok())
            .map(|val| val > 120)
            .unwrap_or(false)
    });
    
    println!("Filtered records (value > 120):");
    processor.print_records(&filtered);
    
    std::fs::remove_file(temp_path)?;
    
    Ok(())
}