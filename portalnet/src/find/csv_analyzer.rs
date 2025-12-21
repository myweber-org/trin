use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub column_types: HashMap<String, String>,
    pub missing_values: usize,
    pub unique_counts: HashMap<String, usize>,
}

pub fn analyze_csv(file_path: &str) -> Result<CsvStats, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let header = match lines.next() {
        Some(Ok(line)) => line,
        _ => return Err("Empty CSV file".into()),
    };

    let columns: Vec<String> = header.split(',').map(|s| s.trim().to_string()).collect();
    let mut column_data: HashMap<String, Vec<String>> = HashMap::new();
    for col in &columns {
        column_data.insert(col.clone(), Vec::new());
    }

    let mut row_count = 0;
    let mut missing_values = 0;

    for line_result in lines {
        let line = line_result?;
        row_count += 1;

        let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if values.len() != columns.len() {
            return Err(format!("Row {} has {} columns, expected {}", 
                row_count, values.len(), columns.len()).into());
        }

        for (i, col) in columns.iter().enumerate() {
            let value = values[i];
            if value.is_empty() {
                missing_values += 1;
            }
            column_data.get_mut(col).unwrap().push(value.to_string());
        }
    }

    let mut column_types = HashMap::new();
    let mut unique_counts = HashMap::new();

    for col in &columns {
        let data = &column_data[col];
        
        let is_numeric = data.iter().all(|v| v.parse::<f64>().is_ok());
        let col_type = if is_numeric { "numeric" } else { "text" };
        column_types.insert(col.clone(), col_type.to_string());

        let unique_count = data.iter().collect::<std::collections::HashSet<_>>().len();
        unique_counts.insert(col.clone(), unique_count);
    }

    Ok(CsvStats {
        row_count,
        column_count: columns.len(),
        column_types,
        missing_values,
        unique_counts,
    })
}

pub fn print_stats(stats: &CsvStats) {
    println!("CSV Analysis Results:");
    println!("Rows: {}", stats.row_count);
    println!("Columns: {}", stats.column_count);
    println!("Missing values: {}", stats.missing_values);
    println!("\nColumn types:");
    for (col, col_type) in &stats.column_types {
        println!("  {}: {}", col, col_type);
    }
    println!("\nUnique values per column:");
    for (col, count) in &stats.unique_counts {
        println!("  {}: {}", col, count);
    }
}