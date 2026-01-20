use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Transaction {
    id: u32,
    customer_id: u32,
    amount: f64,
    category: String,
    timestamp: String,
}

struct TransactionProcessor {
    transactions: Vec<Transaction>,
}

impl TransactionProcessor {
    fn new() -> Self {
        TransactionProcessor {
            transactions: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let transaction: Transaction = result?;
            self.transactions.push(transaction);
        }
        
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Transaction> {
        self.transactions
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }

    fn calculate_total_amount(&self) -> f64 {
        self.transactions.iter().map(|t| t.amount).sum()
    }

    fn calculate_average_amount(&self) -> f64 {
        if self.transactions.is_empty() {
            return 0.0;
        }
        self.calculate_total_amount() / self.transactions.len() as f64
    }

    fn get_customer_summary(&self, customer_id: u32) -> (usize, f64) {
        let customer_transactions: Vec<&Transaction> = self
            .transactions
            .iter()
            .filter(|t| t.customer_id == customer_id)
            .collect();
        
        let total_amount: f64 = customer_transactions.iter().map(|t| t.amount).sum();
        (customer_transactions.len(), total_amount)
    }

    fn save_filtered_to_csv(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        
        let mut wtr = Writer::from_path(output_path)?;
        
        for transaction in filtered {
            wtr.serialize(transaction)?;
        }
        
        wtr.flush()?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = TransactionProcessor::new();
    
    processor.load_from_csv("transactions.csv")?;
    
    println!("Total transactions: {}", processor.transactions.len());
    println!("Total amount: ${:.2}", processor.calculate_total_amount());
    println!("Average transaction: ${:.2}", processor.calculate_average_amount());
    
    let electronics = processor.filter_by_category("electronics");
    println!("Electronics transactions: {}", electronics.len());
    
    let (count, total) = processor.get_customer_summary(123);
    println!("Customer 123: {} transactions, total: ${:.2}", count, total);
    
    processor.save_filtered_to_csv("electronics", "electronics_transactions.csv")?;
    
    Ok(())
}