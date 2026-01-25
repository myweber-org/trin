
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct CsvProcessor {
    data: Vec<Vec<String>>,
    headers: Vec<String>,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();
        let mut data = Vec::new();
        
        for result in rdr.records() {
            let record = result?;
            let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            data.push(row);
        }
        
        Ok(CsvProcessor { data, headers })
    }
    
    pub fn filter_rows<F>(&self, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        self.data.iter()
            .filter(|row| predicate(row))
            .cloned()
            .collect()
    }
    
    pub fn aggregate_column(&self, column_index: usize, operation: &str) -> Option<f64> {
        if column_index >= self.headers.len() {
            return None;
        }
        
        let values: Vec<f64> = self.data.iter()
            .filter_map(|row| row.get(column_index).and_then(|s| s.parse::<f64>().ok()))
            .collect();
        
        if values.is_empty() {
            return None;
        }
        
        match operation {
            "sum" => Some(values.iter().sum()),
            "avg" => Some(values.iter().sum::<f64>() / values.len() as f64),
            "min" => values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            "max" => values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            _ => None,
        }
    }
    
    pub fn get_headers(&self) -> &Vec<String> {
        &self.headers
    }
    
    pub fn row_count(&self) -> usize {
        self.data.len()
    }
    
    pub fn column_count(&self) -> usize {
        self.headers.len()
    }
}

pub fn process_csv_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let processor = CsvProcessor::new(input_path)?;
    
    println!("Processing CSV: {} rows, {} columns", 
             processor.row_count(), 
             processor.column_count());
    
    if processor.column_count() >= 3 {
        if let Some(avg) = processor.aggregate_column(2, "avg") {
            println!("Average of column 3: {:.2}", avg);
        }
    }
    
    let filtered = processor.filter_rows(|row| {
        row.len() > 1 && !row[1].is_empty()
    });
    
    println!("Filtered rows: {}", filtered.len());
    
    let mut wtr = csv::Writer::from_path(output_path)?;
    wtr.write_record(&processor.headers)?;
    
    for row in filtered {
        wtr.write_record(&row)?;
    }
    
    wtr.flush()?;
    Ok(())
}