use rss::Channel;
use std::error::Error;

pub fn fetch_rss_feed(url: &str) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::blocking::get(url)?.bytes()?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

pub fn print_feed_items(channel: &Channel) {
    println!("Feed Title: {}", channel.title());
    println!("Feed Link: {}", channel.link());
    println!("Feed Description: {}", channel.description());
    println!("\n--- Items ---\n");

    for item in channel.items() {
        println!("Item Title: {}", item.title().unwrap_or("No title"));
        println!("Item Link: {}", item.link().unwrap_or("No link"));
        println!("Item Description: {}", item.description().unwrap_or("No description"));
        println!("---");
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let feed_url = "https://example.com/feed.rss";
    let channel = fetch_rss_feed(feed_url)?;
    print_feed_items(&channel);
    Ok(())
}