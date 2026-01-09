
use reqwest;
use quick_xml::de::from_str;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Rss {
    channel: Channel,
}

#[derive(Debug, Deserialize)]
struct Channel {
    title: String,
    item: Vec<Item>,
}

#[derive(Debug, Deserialize)]
struct Item {
    title: String,
    link: String,
    pub_date: Option<String>,
}

async fn fetch_rss(url: &str) -> Result<Rss, Box<dyn Error>> {
    let response = reqwest::get(url).await?.text().await?;
    let rss: Rss = from_str(&response)?;
    Ok(rss)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "https://example.com/feed.rss";
    let rss = fetch_rss(url).await?;
    
    println!("Feed Title: {}", rss.channel.title);
    println!("Latest Items:");
    for item in rss.channel.item.iter().take(5) {
        println!("- {}", item.title);
        if let Some(date) = &item.pub_date {
            println!("  Published: {}", date);
        }
        println!("  Link: {}", item.link);
    }
    
    Ok(())
}