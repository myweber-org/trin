
use reqwest;
use serde::Deserialize;
use std::env;
use log::{info, error};

#[derive(Deserialize, Debug)]
struct WeatherData {
    main: Main,
    name: String,
}

#[derive(Deserialize, Debug)]
struct Main {
    temp: f64,
    humidity: u8,
}

async fn get_weather(api_key: &str, city: &str) -> Result<WeatherData, reqwest::Error> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    let weather: WeatherData = response.json().await?;
    
    Ok(weather)
}

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let api_key = env::var("WEATHER_API_KEY").unwrap_or_else(|_| {
        error!("WEATHER_API_KEY environment variable not set");
        std::process::exit(1);
    });
    
    let city = "London";
    
    match get_weather(&api_key, city).await {
        Ok(weather) => {
            info!("Weather data retrieved for {}", weather.name);
            println!("City: {}", weather.name);
            println!("Temperature: {:.1}°C", weather.main.temp);
            println!("Humidity: {}%", weather.main.humidity);
        }
        Err(e) => {
            error!("Failed to fetch weather data: {}", e);
            std::process::exit(1);
        }
    }
}