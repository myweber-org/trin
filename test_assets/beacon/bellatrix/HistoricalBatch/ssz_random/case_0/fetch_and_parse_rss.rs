use reqwest;
use rss::Channel;
use std::error::Error;

pub async fn fetch_rss_titles(url: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let content = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;

    let titles: Vec<String> = channel.items().iter()
        .filter_map(|item| item.title().map(String::from))
        .collect();

    Ok(titles)
}