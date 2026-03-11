use clap::Parser;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::error::Error;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    owner: String,
    #[arg(short, long)]
    repo: String,
    #[arg(short, long, default_value_t = 30)]
    per_page: u8,
}

#[derive(Deserialize, Debug)]
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
        .query(&[("per_page", args.per_page)])
        .header("User-Agent", "rust-cli-tool")
        .send()?;

    if response.status().is_success() {
        let issues: Vec<Issue> = response.json()?;
        for issue in issues {
            println!(
                "#{} [{}] {} - {}",
                issue.number, issue.state, issue.title, issue.html_url
            );
        }
    } else {
        eprintln!("Failed to fetch issues: {}", response.status());
    }

    Ok(())
}