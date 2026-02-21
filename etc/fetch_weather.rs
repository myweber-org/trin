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
    feels_like: f64,
    humidity: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <city_name>", args[0]);
        std::process::exit(1);
    }
    let city = &args[1];
    let api_key = env::var("OWM_API_KEY").expect("OWM_API_KEY environment variable not set");

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );

    let response = reqwest::get(&url).await?;
    if response.status().is_success() {
        let weather: WeatherData = response.json().await?;
        println!("Weather in {}:", weather.name);
        println!("  Temperature: {:.1}°C", weather.main.temp);
        println!("  Feels like: {:.1}°C", weather.main.feels_like);
        println!("  Humidity: {}%", weather.main.humidity);
    } else {
        eprintln!("Failed to fetch weather data. City not found or API error.");
        std::process::exit(1);
    }

    Ok(())
}