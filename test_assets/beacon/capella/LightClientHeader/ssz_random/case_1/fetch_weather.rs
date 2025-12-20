use reqwest;
use serde::Deserialize;
use std::env;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENWEATHER_API_KEY")
        .expect("Please set the OPENWEATHER_API_KEY environment variable");
    
    let args: Vec<String> = env::args().collect();
    let city = if args.len() > 1 {
        args[1].clone()
    } else {
        "London".to_string()
    };

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );

    let response = reqwest::get(&url).await?;
    
    if response.status().is_success() {
        let weather: WeatherData = response.json().await?;
        println!("Weather in {}:", weather.name);
        println!("Temperature: {:.1}°C", weather.main.temp);
        println!("Humidity: {}%", weather.main.humidity);
    } else {
        eprintln!("Failed to fetch weather data. Status: {}", response.status());
    }

    Ok(())
}