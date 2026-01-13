use reqwest;
use rss::Channel;
use std::error::Error;

pub async fn fetch_and_parse_rss(feed_url: &str) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get(feed_url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_rss() {
        let url = "https://example.com/feed.rss";
        let result = fetch_and_parse_rss(url).await;
        assert!(result.is_ok());
    }
}