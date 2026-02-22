use std::error::Error;
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct WeatherData {
    city: String,
    temperature: f64,
    condition: String,
    humidity: u8,
}

async fn fetch_weather(city: &str) -> Result<WeatherData, Box<dyn Error>> {
    let url = format!("https://api.example.com/weather?city={}", city);
    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err("Failed to fetch weather data".into());
    }
    
    let weather: WeatherData = response.json().await?;
    Ok(weather)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <city>", args[0]);
        std::process::exit(1);
    }
    
    let city = &args[1];
    match fetch_weather(city).await {
        Ok(data) => {
            println!("Weather in {}:", data.city);
            println!("  Temperature: {:.1}°C", data.temperature);
            println!("  Condition: {}", data.condition);
            println!("  Humidity: {}%", data.humidity);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    
    Ok(())
}