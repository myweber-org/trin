use std::error::Error;
use std::fs::File;
use csv::Reader;

pub struct CsvSummary {
    pub row_count: usize,
    pub column_count: usize,
    pub headers: Vec<String>,
}

pub fn analyze_csv(file_path: &str) -> Result<CsvSummary, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = Reader::from_reader(file);
    
    let headers = rdr.headers()?.iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    
    let mut row_count = 0;
    for result in rdr.records() {
        let _record = result?;
        row_count += 1;
    }
    
    Ok(CsvSummary {
        row_count,
        column_count: headers.len(),
        headers,
    })
}

pub fn print_summary(summary: &CsvSummary) {
    println!("CSV Analysis Summary:");
    println!("Rows: {}", summary.row_count);
    println!("Columns: {}", summary.column_count);
    println!("Headers: {:?}", summary.headers);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_analyze_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let summary = analyze_csv(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(summary.row_count, 2);
        assert_eq!(summary.column_count, 3);
        assert_eq!(summary.headers, vec!["name", "age", "city"]);
    }
}