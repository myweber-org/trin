use clap::Parser;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::error::Error;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    owner: String,
    #[arg(short, long)]
    repo: String,
    #[arg(short, long, default_value_t = 10)]
    limit: u8,
}

#[derive(Deserialize)]
struct Issue {
    number: u64,
    title: String,
    state: String,
    html_url: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let client = Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/{}/issues",
        args.owner, args.repo
    );

    let response = client
        .get(&url)
        .header("User-Agent", "rust-cli-tool")
        .query(&[("per_page", args.limit)])
        .send()?;

    if response.status().is_success() {
        let issues: Vec<Issue> = response.json()?;
        for issue in issues {
            println!("#{} [{}] {}", issue.number, issue.state, issue.title);
            println!("   {}", issue.html_url);
            println!();
        }
    } else {
        eprintln!("Failed to fetch issues: {}", response.status());
    }

    Ok(())
}