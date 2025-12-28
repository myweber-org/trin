use reqwest;
use rss::Channel;
use std::error::Error;

pub async fn fetch_rss_feed(url: &str) -> Result<Vec<RssItem>, Box<dyn Error>> {
    let body = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(&body[..])?;

    let items: Vec<RssItem> = channel
        .items()
        .iter()
        .map(|item| RssItem {
            title: item.title().unwrap_or("").to_string(),
            link: item.link().unwrap_or("").to_string(),
            description: item.description().unwrap_or("").to_string(),
            pub_date: item.pub_date().unwrap_or("").to_string(),
        })
        .collect();

    Ok(items)
}

#[derive(Debug)]
pub struct RssItem {
    pub title: String,
    pub link: String,
    pub description: String,
    pub pub_date: String,
}