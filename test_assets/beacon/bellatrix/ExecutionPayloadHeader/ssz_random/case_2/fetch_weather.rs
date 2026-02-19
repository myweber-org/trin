use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::Deserialize;
use reqwest::Error;

#[derive(Deserialize, Debug)]
struct WeatherData {
    temperature: f64,
    humidity: f64,
    conditions: String,
}

struct WeatherCache {
    cache: HashMap<String, (WeatherData, Instant)>,
    ttl: Duration,
}

impl WeatherCache {
    fn new(ttl_seconds: u64) -> Self {
        WeatherCache {
            cache: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    async fn get_weather(&mut self, city: &str, api_key: &str) -> Result<WeatherData, Error> {
        if let Some((data, timestamp)) = self.cache.get(city) {
            if timestamp.elapsed() < self.ttl {
                return Ok(data.clone());
            }
        }

        let url = format!(
            "https://api.weather.example.com/data?city={}&key={}",
            city, api_key
        );
        let response = reqwest::get(&url).await?;
        let weather_data: WeatherData = response.json().await?;

        self.cache.insert(city.to_string(), (weather_data.clone(), Instant::now()));
        Ok(weather_data)
    }

    fn clear_expired(&mut self) {
        let now = Instant::now();
        self.cache.retain(|_, (_, timestamp)| now.duration_since(*timestamp) < self.ttl);
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut cache = WeatherCache::new(300);
    let api_key = "your_api_key_here";

    match cache.get_weather("London", api_key).await {
        Ok(weather) => {
            println!("Temperature: {}°C", weather.temperature);
            println!("Humidity: {}%", weather.humidity);
            println!("Conditions: {}", weather.conditions);
        }
        Err(e) => eprintln!("Failed to fetch weather: {}", e),
    }

    cache.clear_expired();
    Ok(())
}