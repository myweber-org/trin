use reqwest;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repo_owner = "rust-lang";
    let repo_name = "rust";
    let url = format!("https://api.github.com/repos/{}/{}/issues", repo_owner, repo_name);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-API-Client")
        .send()
        .await?;

    if response.status().is_success() {
        let issues: Value = response.json().await?;
        if let Some(issues_array) = issues.as_array() {
            println!("Recent issues for {}/{}:", repo_owner, repo_name);
            for issue in issues_array.iter().take(5) {
                let title = issue["title"].as_str().unwrap_or("No title");
                let number = issue["number"].as_u64().unwrap_or(0);
                let state = issue["state"].as_str().unwrap_or("unknown");
                println!("#{} [{}] {}", number, state, title);
            }
        } else {
            println!("No issues found or invalid response format.");
        }
    } else {
        println!("Failed to fetch issues. Status: {}", response.status());
    }

    Ok(())
}