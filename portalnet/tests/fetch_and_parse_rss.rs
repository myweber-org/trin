extern crate rss;

use std::error::Error;
use std::io::BufReader;
use std::fs::File;

fn read_rss_from_file(path: &str) -> Result<rss::Channel, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let channel = rss::Channel::read_from(reader)?;
    Ok(channel)
}

fn main() -> Result<(), Box<dyn Error>> {
    let channel = read_rss_from_file("sample_feed.xml")?;
    println!("Channel title: {}", channel.title());
    println!("Number of items: {}", channel.items().len());
    for item in channel.items().iter().take(3) {
        if let Some(title) = item.title() {
            println!("Item title: {}", title);
        }
    }
    Ok(())
}