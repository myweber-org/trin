use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct ColumnStats {
    name: String,
    count: usize,
    sum: f64,
    min: f64,
    max: f64,
}

impl ColumnStats {
    fn new(name: &str) -> Self {
        ColumnStats {
            name: name.to_string(),
            count: 0,
            sum: 0.0,
            min: f64::MAX,
            max: f64::MIN,
        }
    }

    fn update(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    fn average(&self) -> f64 {
        if self.count > 0 {
            self.sum / self.count as f64
        } else {
            0.0
        }
    }
}

fn analyze_csv(file_path: &str) -> Result<Vec<ColumnStats>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let header = match lines.next() {
        Some(Ok(line)) => line,
        _ => return Err("Empty file or missing header".into()),
    };

    let column_names: Vec<&str> = header.split(',').collect();
    let mut stats: Vec<ColumnStats> = column_names
        .iter()
        .map(|&name| ColumnStats::new(name))
        .collect();

    for line_result in lines {
        let line = line_result?;
        let values: Vec<&str> = line.split(',').collect();

        if values.len() != column_names.len() {
            continue;
        }

        for (i, value_str) in values.iter().enumerate() {
            if let Ok(value) = value_str.parse::<f64>() {
                stats[i].update(value);
            }
        }
    }

    Ok(stats)
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = "data.csv";
    let column_stats = analyze_csv(file_path)?;

    for stat in column_stats {
        if stat.count > 0 {
            println!(
                "Column: {}, Count: {}, Avg: {:.2}, Min: {:.2}, Max: {:.2}",
                stat.name,
                stat.count,
                stat.average(),
                stat.min,
                stat.max
            );
        }
    }

    Ok(())
}