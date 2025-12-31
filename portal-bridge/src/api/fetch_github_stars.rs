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
}

#[derive(Deserialize)]
struct RepoInfo {
    stargazers_count: u64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let client = Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/{}",
        args.owner, args.repo
    );

    let response = client
        .get(&url)
        .header("User-Agent", "github-stars-fetcher")
        .send()?;

    if response.status().is_success() {
        let repo_info: RepoInfo = response.json()?;
        println!("{} has {} stars", args.repo, repo_info.stargazers_count);
    } else {
        eprintln!("Failed to fetch repository info. Status: {}", response.status());
    }

    Ok(())
}