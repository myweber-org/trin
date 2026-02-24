use std::collections::HashMap;
use std::time::{Duration, Instant};
use reqwest::Error;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    main: MainData,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MainData {
    temp: f64,
    humidity: u8,
}

struct WeatherCache {
    data: HashMap<String, (WeatherResponse, Instant)>,
    ttl: Duration,
}

impl WeatherCache {
    fn new(ttl_seconds: u64) -> Self {
        WeatherCache {
            data: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    async fn get_weather(&mut self, city: &str, api_key: &str) -> Result<WeatherResponse, Error> {
        let now = Instant::now();
        
        if let Some((cached_data, timestamp)) = self.data.get(city) {
            if now.duration_since(*timestamp) < self.ttl {
                println!("Returning cached data for {}", city);
                return Ok(cached_data.clone());
            }
        }

        println!("Fetching fresh data for {}", city);
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, api_key
        );
        
        let response = reqwest::get(&url).await?;
        let weather_data: WeatherResponse = response.json().await?;
        
        self.data.insert(city.to_string(), (weather_data.clone(), now));
        Ok(weather_data)
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let api_key = "your_api_key_here";
    let mut cache = WeatherCache::new(300);
    
    let cities = ["London", "Paris", "Tokyo"];
    
    for city in cities.iter() {
        match cache.get_weather(city, api_key).await {
            Ok(data) => {
                println!("Weather in {}: {:.1}°C, {}% humidity", 
                         data.name, data.main.temp, data.main.humidity);
            }
            Err(e) => {
                eprintln!("Failed to fetch weather for {}: {}", city, e);
            }
        }
    }
    
    Ok(())
}