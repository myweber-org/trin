use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct GitHubIssue {
    number: u64,
    title: String,
    state: String,
    html_url: String,
    user: GitHubUser,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubUser {
    login: String,
}

async fn fetch_open_issues(owner: &str, repo: &str) -> Result<Vec<GitHubIssue>, Box<dyn Error>> {
    let url = format!("https://api.github.com/repos/{}/{}/issues", owner, repo);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-API-Client")
        .query(&[("state", "open")])
        .send()
        .await?;

    if response.status().is_success() {
        let issues: Vec<GitHubIssue> = response.json().await?;
        Ok(issues)
    } else {
        Err(format!("Failed to fetch issues: {}", response.status()).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let owner = "rust-lang";
    let repo = "rust";

    match fetch_open_issues(owner, repo).await {
        Ok(issues) => {
            println!("Open issues for {}/{}:", owner, repo);
            for issue in issues {
                println!("#{} - {} ({})", issue.number, issue.title, issue.state);
                println!("URL: {}", issue.html_url);
                println!("Reported by: {}", issue.user.login);
                println!("---");
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}