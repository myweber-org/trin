
use reqwest;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct GitHubUser {
    login: String,
    id: u64,
    avatar_url: String,
    html_url: String,
    name: Option<String>,
    company: Option<String>,
    blog: Option<String>,
    location: Option<String>,
    public_repos: u32,
    followers: u32,
    following: u32,
}

async fn fetch_github_user(username: &str) -> Result<GitHubUser, reqwest::Error> {
    let url = format!("https://api.github.com/users/{}", username);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-GitHub-API-Client")
        .send()
        .await?;
    
    let user: GitHubUser = response.json().await?;
    Ok(user)
}

#[tokio::main]
async fn main() {
    match fetch_github_user("torvalds").await {
        Ok(user) => {
            println!("User: {}", user.login);
            println!("Name: {:?}", user.name);
            println!("Followers: {}", user.followers);
            println!("Public Repos: {}", user.public_repos);
            println!("Profile: {}", user.html_url);
        }
        Err(e) => eprintln!("Error fetching user: {}", e),
    }
}