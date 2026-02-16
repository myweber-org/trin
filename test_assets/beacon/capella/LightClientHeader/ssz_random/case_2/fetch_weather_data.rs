
use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct WeatherData {
    main: Main,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Main {
    temp: f64,
    feels_like: f64,
    humidity: u8,
}

pub async fn get_weather(api_key: &str, city: &str) -> Result<WeatherData, Box<dyn Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );

    let response = reqwest::get(&url).await?;
    let weather: WeatherData = response.json().await?;

    Ok(weather)
}

pub fn display_weather(data: &WeatherData) {
    println!("Weather in {}:", data.name);
    println!("  Temperature: {:.1}°C", data.main.temp);
    println!("  Feels like: {:.1}°C", data.main.feels_like);
    println!("  Humidity: {}%", data.main.humidity);
}use std::error::Error;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct WeatherData {
    location: String,
    temperature: f64,
    condition: String,
    humidity: u8,
}

fn fetch_weather(city: &str) -> Result<WeatherData, Box<dyn Error>> {
    let mock_response = format!(
        r#"{{
            "location": "{}",
            "temperature": 22.5,
            "condition": "Sunny",
            "humidity": 65
        }}"#,
        city
    );

    let weather: WeatherData = serde_json::from_str(&mock_response)?;
    Ok(weather)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let city = args.get(1).map(|s| s.as_str()).unwrap_or("London");

    let weather = fetch_weather(city)?;
    
    println!("Weather in {}:", weather.location);
    println!("  Temperature: {:.1}°C", weather.temperature);
    println!("  Condition: {}", weather.condition);
    println!("  Humidity: {}%", weather.humidity);

    Ok(())
}