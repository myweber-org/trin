
use std::collections::HashMap;

pub fn clean_numeric_data(
    data: &[HashMap<String, String>],
    numeric_fields: &[&str],
) -> Vec<HashMap<String, f64>> {
    data.iter()
        .filter_map(|record| {
            let mut cleaned = HashMap::new();
            
            for &field in numeric_fields {
                if let Some(value) = record.get(field) {
                    match value.trim().parse::<f64>() {
                        Ok(num) => {
                            cleaned.insert(field.to_string(), num);
                        }
                        Err(_) => {
                            return None;
                        }
                    }
                } else {
                    return None;
                }
            }
            
            Some(cleaned)
        })
        .collect()
}

pub fn calculate_averages(cleaned_data: &[HashMap<String, f64>]) -> HashMap<String, f64> {
    let mut sums: HashMap<String, f64> = HashMap::new();
    let mut counts: HashMap<String, usize> = HashMap::new();
    
    for record in cleaned_data {
        for (key, value) in record {
            *sums.entry(key.clone()).or_insert(0.0) += value;
            *counts.entry(key.clone()).or_insert(0) += 1;
        }
    }
    
    sums.iter()
        .map(|(key, sum)| {
            let count = counts.get(key).unwrap_or(&1);
            (key.clone(), sum / *count as f64)
        })
        .collect()
}