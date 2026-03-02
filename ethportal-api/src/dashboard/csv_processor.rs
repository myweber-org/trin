use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn parse_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            continue;
        }

        let id = parts[0].parse::<u32>()?;
        let name = parts[1].to_string();
        let value = parts[2].parse::<f64>()?;
        let category = parts[3].to_string();

        records.push(Record {
            id,
            name,
            value,
            category,
        });
    }

    Ok(records)
}

fn aggregate_by_category(records: &[Record]) -> Vec<(String, f64, usize)> {
    use std::collections::HashMap;

    let mut aggregation: HashMap<String, (f64, usize)> = HashMap::new();

    for record in records {
        let entry = aggregation
            .entry(record.category.clone())
            .or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }

    aggregation
        .into_iter()
        .map(|(category, (total, count))| (category, total, count))
        .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let records = parse_csv("data.csv")?;
    let aggregated = aggregate_by_category(&records);

    for (category, total, count) in aggregated {
        println!(
            "Category: {}, Total Value: {:.2}, Record Count: {}",
            category, total, count
        );
    }

    Ok(())
}