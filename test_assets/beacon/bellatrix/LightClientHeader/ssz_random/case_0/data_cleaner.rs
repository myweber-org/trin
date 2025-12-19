use std::collections::HashSet;
use std::io::{self, BufRead};

pub fn clean_data(input: Vec<String>) -> Vec<String> {
    let mut unique_items: HashSet<String> = input.into_iter().collect();
    let mut sorted_items: Vec<String> = unique_items.into_iter().collect();
    sorted_items.sort();
    sorted_items
}

fn main() {
    println!("Enter data lines (press Ctrl+D when finished):");
    let stdin = io::stdin();
    let input_lines: Vec<String> = stdin.lock().lines().filter_map(Result::ok).collect();
    
    let cleaned = clean_data(input_lines);
    println!("Cleaned and sorted data:");
    for item in cleaned {
        println!("{}", item);
    }
}