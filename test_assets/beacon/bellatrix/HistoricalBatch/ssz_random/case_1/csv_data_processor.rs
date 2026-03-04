
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

pub struct CsvProcessor {
    records: Vec<Record>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.len() != 4 {
                continue;
            }

            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let active = parts[3].parse::<bool>().unwrap_or(false);

            self.records.push(Record {
                id,
                name,
                value,
                active,
            });
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_by_value(&self, threshold: f64) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.value >= threshold && r.active)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn export_filtered<P: AsRef<Path>>(
        &self,
        threshold: f64,
        output_path: P,
    ) -> io::Result<usize> {
        let filtered = self.filter_by_value(threshold);
        let mut file = File::create(output_path)?;

        writeln!(file, "id,name,value,active")?;
        let mut count = 0;

        for record in filtered {
            writeln!(
                file,
                "{},{},{},{}",
                record.id, record.name, record.value, record.active
            )?;
            count += 1;
        }

        Ok(count)
    }

    pub fn get_statistics(&self) -> (usize, Option<f64>, usize) {
        let total = self.records.len();
        let average = self.calculate_average();
        let active_count = self.records.iter().filter(|r| r.active).count();

        (total, average, active_count)
    }
}

pub fn process_csv_data(input_path: &str, output_path: &str, threshold: f64) -> Result<(), Box<dyn Error>> {
    let mut processor = CsvProcessor::new();
    let loaded = processor.load_from_file(input_path)?;

    if loaded == 0 {
        return Err("No valid records loaded from input file".into());
    }

    let stats = processor.get_statistics();
    println!("Loaded {} records", stats.0);
    println!("Average value: {:.2}", stats.1.unwrap_or(0.0));
    println!("Active records: {}", stats.2);

    let exported = processor.export_filtered(threshold, output_path)?;
    println!("Exported {} records meeting threshold {}", exported, threshold);

    Ok(())
}