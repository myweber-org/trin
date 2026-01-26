use std::collections::HashSet;
use std::io::{self, BufRead, Write};

fn clean_data(input: Vec<String>) -> Vec<String> {
    let mut unique_items: HashSet<String> = HashSet::new();
    for item in input {
        unique_items.insert(item);
    }
    
    let mut sorted_items: Vec<String> = unique_items.into_iter().collect();
    sorted_items.sort();
    sorted_items
}

fn main() {
    println!("Enter data lines (empty line to finish):");
    
    let stdin = io::stdin();
    let mut input_lines = Vec::new();
    
    for line in stdin.lock().lines() {
        match line {
            Ok(content) => {
                if content.is_empty() {
                    break;
                }
                input_lines.push(content);
            }
            Err(_) => break,
        }
    }
    
    let cleaned = clean_data(input_lines);
    
    println!("Cleaned and sorted data:");
    for item in cleaned {
        println!("{}", item);
    }
}