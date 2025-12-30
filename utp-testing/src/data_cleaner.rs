use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub struct DataCleaner {
    min_value: f64,
    max_value: f64,
    normalize_range: (f64, f64),
}

impl DataCleaner {
    pub fn new(min_value: f64, max_value: f64, normalize_range: (f64, f64)) -> Self {
        DataCleaner {
            min_value,
            max_value,
            normalize_range,
        }
    }

    pub fn filter_and_normalize(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(output_path)?;

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                continue;
            }

            if let Ok(value) = trimmed.parse::<f64>() {
                if value >= self.min_value && value <= self.max_value {
                    let normalized = self.normalize_value(value);
                    writeln!(output_file, "{:.6}", normalized)?;
                }
            }
        }

        Ok(())
    }

    fn normalize_value(&self, value: f64) -> f64 {
        let (target_min, target_max) = self.normalize_range;
        let normalized = (value - self.min_value) / (self.max_value - self.min_value);
        normalized * (target_max - target_min) + target_min
    }
}

pub fn process_dataset(input_file: &str, output_file: &str) -> Result<(), Box<dyn Error>> {
    let cleaner = DataCleaner::new(0.0, 100.0, (0.0, 1.0));
    cleaner.filter_and_normalize(input_file, output_file)?;
    
    println!("Data processing completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_normalize_value() {
        let cleaner = DataCleaner::new(0.0, 100.0, (0.0, 1.0));
        assert_eq!(cleaner.normalize_value(50.0), 0.5);
        assert_eq!(cleaner.normalize_value(0.0), 0.0);
        assert_eq!(cleaner.normalize_value(100.0), 1.0);
    }

    #[test]
    fn test_filter_and_normalize() -> Result<(), Box<dyn Error>> {
        let mut input_file = NamedTempFile::new()?;
        writeln!(input_file, "10.5\n")?;
        writeln!(input_file, "150.0\n")?;
        writeln!(input_file, "-5.0\n")?;
        writeln!(input_file, "75.3\n")?;
        writeln!(input_file, "invalid\n")?;
        writeln!(input_file, "   \n")?;
        writeln!(input_file, "99.9\n")?;

        let output_file = NamedTempFile::new()?;
        
        let cleaner = DataCleaner::new(0.0, 100.0, (0.0, 1.0));
        cleaner.filter_and_normalize(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        )?;

        let output_content = std::fs::read_to_string(output_file.path())?;
        let lines: Vec<&str> = output_content.trim().split('\n').collect();
        
        assert_eq!(lines.len(), 3);
        assert!(lines.contains(&"0.105000"));
        assert!(lines.contains(&"0.753000"));
        assert!(lines.contains(&"0.999000"));

        Ok(())
    }
}