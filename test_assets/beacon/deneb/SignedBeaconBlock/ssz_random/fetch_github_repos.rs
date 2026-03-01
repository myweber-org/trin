use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct Repository {
    name: String,
    description: Option<String>,
    html_url: String,
    stargazers_count: u32,
}

async fn fetch_repositories(username: &str) -> Result<Vec<Repository>, Box<dyn Error>> {
    let url = format!("https://api.github.com/users/{}/repos", username);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-GitHub-Client")
        .send()
        .await?;

    if response.status().is_success() {
        let repos: Vec<Repository> = response.json().await?;
        Ok(repos)
    } else {
        Err(format!("Failed to fetch repositories: {}", response.status()).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let username = "rust-lang";
    println!("Fetching repositories for user: {}", username);
    
    match fetch_repositories(username).await {
        Ok(repos) => {
            println!("Found {} repositories:", repos.len());
            for repo in repos {
                println!("- {} ({})", repo.name, repo.html_url);
                if let Some(desc) = repo.description {
                    println!("  Description: {}", desc);
                }
                println!("  Stars: {}", repo.stargazers_count);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    
    Ok(())
}