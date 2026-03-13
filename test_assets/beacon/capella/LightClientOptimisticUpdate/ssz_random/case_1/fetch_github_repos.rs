use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Repository {
    name: String,
    full_name: String,
    html_url: String,
    description: Option<String>,
    stargazers_count: u32,
    forks_count: u32,
}

async fn fetch_user_repos(username: &str) -> Result<Vec<Repository>, Box<dyn Error>> {
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
    match fetch_user_repos(username).await {
        Ok(repos) => {
            println!("Public repositories for '{}':", username);
            for repo in repos.iter().take(5) {
                println!("- {} (⭐ {} | 🍴 {})", repo.name, repo.stargazers_count, repo.forks_count);
                if let Some(desc) = &repo.description {
                    println!("  Description: {}", desc);
                }
                println!("  URL: {}", repo.html_url);
                println!();
            }
            println!("Total repositories fetched: {}", repos.len());
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    Ok(())
}