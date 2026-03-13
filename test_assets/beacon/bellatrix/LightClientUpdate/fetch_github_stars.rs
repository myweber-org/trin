
use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize)]
struct RepoInfo {
    stargazers_count: u64,
}

pub async fn get_star_count(owner: &str, repo: &str) -> Result<u64, Box<dyn Error>> {
    let url = format!("https://api.github.com/repos/{}/{}", owner, repo);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-GitHub-Stars-Fetcher")
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(format!("GitHub API error: {}", response.status()).into());
    }
    
    let repo_info: RepoInfo = response.json().await?;
    Ok(repo_info.stargazers_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn test_get_star_count_success() {
        let _m = mock("GET", "/repos/rust-lang/rust")
            .match_header("User-Agent", Matcher::Exact("Rust-GitHub-Stars-Fetcher".to_string()))
            .with_status(200)
            .with_body(r#"{"stargazers_count": 123456}"#)
            .create();

        let stars = get_star_count("rust-lang", "rust").await.unwrap();
        assert_eq!(stars, 123456);
    }

    #[tokio::test]
    async fn test_get_star_count_not_found() {
        let _m = mock("GET", "/repos/invalid/notfound")
            .with_status(404)
            .create();

        let result = get_star_count("invalid", "notfound").await;
        assert!(result.is_err());
    }
}