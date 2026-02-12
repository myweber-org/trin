use std::error::Error;
use reqwest;
use rss::Channel;

pub fn fetch_rss_feed(url: &str) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::blocking::get(url)?.bytes()?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

fn main() -> Result<(), Box<dyn Error>> {
    let url = "https://example.com/feed.rss";
    let channel = fetch_rss_feed(url)?;

    println!("Feed Title: {}", channel.title());
    println!("Feed Description: {}", channel.description());
    println!("\nLatest Items:");
    for item in channel.items().iter().take(5) {
        if let Some(title) = item.title() {
            println!("- {}", title);
        }
    }
    Ok(())
}