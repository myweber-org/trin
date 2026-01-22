
use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct WeatherData {
    temperature: f64,
    humidity: f64,
    wind_speed: f64,
    description: String,
}

async fn fetch_weather_data(api_key: &str, city: &str) -> Result<WeatherData, Box<dyn Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );

    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }

    let json_data: serde_json::Value = response.json().await?;
    
    let weather_data = WeatherData {
        temperature: json_data["main"]["temp"].as_f64().unwrap_or(0.0),
        humidity: json_data["main"]["humidity"].as_f64().unwrap_or(0.0),
        wind_speed: json_data["wind"]["speed"].as_f64().unwrap_or(0.0),
        description: json_data["weather"][0]["description"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string(),
    };

    Ok(weather_data)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = "your_api_key_here";
    let city = "London";
    
    match fetch_weather_data(api_key, city).await {
        Ok(data) => {
            println!("Weather in {}:", city);
            println!("Temperature: {:.1}°C", data.temperature);
            println!("Humidity: {:.1}%", data.humidity);
            println!("Wind Speed: {:.1} m/s", data.wind_speed);
            println!("Description: {}", data.description);
        }
        Err(e) => eprintln!("Failed to fetch weather data: {}", e),
    }
    
    Ok(())
}