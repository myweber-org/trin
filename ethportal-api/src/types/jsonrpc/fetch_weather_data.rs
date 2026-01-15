
use reqwest::Error;
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
async fn main() -> Result<(), Error> {
    let api_key = env::var("OWM_API_KEY").expect("OWM_API_KEY not set");
    let city = env::args().nth(1).unwrap_or_else(|| "London".to_string());

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );

    let response = reqwest::get(&url).await?;
    let weather: WeatherData = response.json().await?;

    println!("Weather in {}:", weather.name);
    println!("Temperature: {:.1}°C", weather.main.temp);
    println!("Humidity: {}%", weather.main.humidity);

    Ok(())
}