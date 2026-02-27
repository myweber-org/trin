use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct CsvStats {
    row_count: usize,
    column_count: usize,
    numeric_columns: Vec<usize>,
}

fn analyze_csv(file_path: &str) -> Result<CsvStats, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    let header = match lines.next() {
        Some(Ok(line)) => line,
        _ => return Err("Empty CSV file".into()),
    };
    
    let column_count = header.split(',').count();
    let mut row_count = 1;
    let mut numeric_column_flags = vec![true; column_count];
    
    for line_result in lines {
        let line = line_result?;
        row_count += 1;
        
        let values: Vec<&str> = line.split(',').collect();
        if values.len() != column_count {
            return Err("Inconsistent column count".into());
        }
        
        for (i, value) in values.iter().enumerate() {
            if numeric_column_flags[i] && value.trim().parse::<f64>().is_err() {
                numeric_column_flags[i] = false;
            }
        }
    }
    
    let numeric_columns: Vec<usize> = numeric_column_flags
        .iter()
        .enumerate()
        .filter(|(_, &is_numeric)| is_numeric)
        .map(|(idx, _)| idx)
        .collect();
    
    Ok(CsvStats {
        row_count,
        column_count,
        numeric_columns,
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <csv_file>", args[0]);
        std::process::exit(1);
    }
    
    let stats = analyze_csv(&args[1])?;
    println!("CSV Analysis Results:");
    println!("Total rows: {}", stats.row_count);
    println!("Total columns: {}", stats.column_count);
    println!("Numeric column indices: {:?}", stats.numeric_columns);
    
    Ok(())
}