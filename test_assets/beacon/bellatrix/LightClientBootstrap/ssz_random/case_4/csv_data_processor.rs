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

    fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut reader = Reader::from_reader(file);
        
        for result in reader.deserialize() {
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
        self.transactions
            .iter()
            .map(|t| t.amount)
            .sum()
    }

    fn calculate_average_amount(&self) -> f64 {
        if self.transactions.is_empty() {
            return 0.0;
        }
        self.calculate_total_amount() / self.transactions.len() as f64
    }

    fn find_largest_transaction(&self) -> Option<&Transaction> {
        self.transactions
            .iter()
            .max_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap())
    }

    fn save_filtered_transactions(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let file = File::create(output_path)?;
        let mut writer = Writer::from_writer(file);
        
        for transaction in filtered {
            writer.serialize(transaction)?;
        }
        
        writer.flush()?;
        Ok(())
    }

    fn generate_summary_report(&self) -> String {
        let total = self.calculate_total_amount();
        let average = self.calculate_average_amount();
        let largest = self.find_largest_transaction();
        
        let mut report = format!(
            "Transaction Summary Report\n\
            ==========================\n\
            Total Transactions: {}\n\
            Total Amount: ${:.2}\n\
            Average Amount: ${:.2}\n",
            self.transactions.len(),
            total,
            average
        );
        
        if let Some(largest_tx) = largest {
            report.push_str(&format!(
                "Largest Transaction: ID {} - ${:.2} ({})\n",
                largest_tx.id, largest_tx.amount, largest_tx.category
            ));
        }
        
        report
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = TransactionProcessor::new();
    
    processor.load_from_file("transactions.csv")?;
    
    println!("{}", processor.generate_summary_report());
    
    let electronics_transactions = processor.filter_by_category("Electronics");
    println!("Found {} electronics transactions", electronics_transactions.len());
    
    processor.save_filtered_transactions("Electronics", "electronics_transactions.csv")?;
    
    Ok(())
}