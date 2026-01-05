
use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

fn process_csv(input_path: &str, output_path: &str, min_value: f64) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut wtr = Writer::from_path(output_path)?;

    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.value >= min_value && record.active {
            wtr.serialize(&record)?;
        }
    }

    wtr.flush()?;
    Ok(())
}

fn generate_sample_csv(path: &str) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(path)?;
    let records = vec![
        Record { id: 1, name: String::from("Alpha"), value: 42.5, active: true },
        Record { id: 2, name: String::from("Beta"), value: 18.3, active: false },
        Record { id: 3, name: String::from("Gamma"), value: 75.1, active: true },
        Record { id: 4, name: String::from("Delta"), value: 9.7, active: true },
    ];

    for record in records {
        wtr.serialize(&record)?;
    }
    wtr.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "input_data.csv";
    let output_file = "filtered_data.csv";
    let threshold = 20.0;

    generate_sample_csv(input_file)?;
    process_csv(input_file, output_file, threshold)?;

    println!("Processing completed. Filtered data saved to {}", output_file);
    Ok(())
}