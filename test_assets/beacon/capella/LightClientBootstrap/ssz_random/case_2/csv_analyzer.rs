use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub has_header: bool,
    pub sample_data: Vec<Vec<String>>,
}

pub fn analyze_csv(file_path: &str, sample_size: usize) -> Result<CsvStats, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let headers = rdr.headers()?.clone();
    let has_header = !headers.is_empty();
    let column_count = headers.len();
    
    let mut row_count = 0;
    let mut sample_data = Vec::with_capacity(sample_size);
    
    for result in rdr.records() {
        let record = result?;
        row_count += 1;
        
        if sample_data.len() < sample_size {
            let row_data: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            sample_data.push(row_data);
        }
    }
    
    Ok(CsvStats {
        row_count,
        column_count,
        has_header,
        sample_data,
    })
}

pub fn display_stats(stats: &CsvStats) {
    println!("CSV Analysis Results:");
    println!("Rows: {}", stats.row_count);
    println!("Columns: {}", stats.column_count);
    println!("Has header: {}", stats.has_header);
    
    if !stats.sample_data.is_empty() {
        println!("\nSample data (first {} rows):", stats.sample_data.len());
        for (i, row) in stats.sample_data.iter().enumerate() {
            println!("Row {}: {:?}", i + 1, row);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_analyze_csv() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "name,age,city")?;
        writeln!(temp_file, "Alice,30,New York")?;
        writeln!(temp_file, "Bob,25,London")?;
        writeln!(temp_file, "Charlie,35,Tokyo")?;
        
        let stats = analyze_csv(temp_file.path().to_str().unwrap(), 2)?;
        
        assert_eq!(stats.row_count, 3);
        assert_eq!(stats.column_count, 3);
        assert!(stats.has_header);
        assert_eq!(stats.sample_data.len(), 2);
        
        Ok(())
    }
}