use std::collections::HashMap;
use std::time::{Duration, Instant};
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct WeatherResponse {
    main: MainData,
    name: String,
}

#[derive(Deserialize, Debug)]
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

    fn get_weather(&mut self, city: &str, api_key: &str) -> Result<&WeatherResponse, String> {
        let now = Instant::now();
        
        if let Some((cached_data, timestamp)) = self.data.get(city) {
            if now.duration_since(*timestamp) < self.ttl {
                return Ok(cached_data);
            }
        }

        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, api_key
        );

        let client = Client::new();
        let response = client.get(&url).send().map_err(|e| e.to_string())?;
        
        if !response.status().is_success() {
            return Err(format!("API request failed: {}", response.status()));
        }

        let weather_data: WeatherResponse = response.json().map_err(|e| e.to_string())?;
        self.data.insert(city.to_string(), (weather_data, now));
        
        Ok(&self.data.get(city).unwrap().0)
    }
}

fn main() {
    let api_key = std::env::var("WEATHER_API_KEY").unwrap_or_else(|_| "demo_key".to_string());
    let mut cache = WeatherCache::new(300);
    
    match cache.get_weather("London", &api_key) {
        Ok(weather) => {
            println!("Weather in {}: {:.1}°C, {}% humidity", 
                     weather.name, weather.main.temp, weather.main.humidity);
        }
        Err(e) => {
            eprintln!("Failed to fetch weather: {}", e);
        }
    }
}