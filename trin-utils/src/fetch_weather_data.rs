
use reqwest;
use serde_json::Value;

pub async fn fetch_weather_data(api_key: &str, city: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    let json: Value = response.json().await?;
    
    if let Some(temp) = json["main"]["temp"].as_f64() {
        Ok(temp)
    } else {
        Err("Failed to parse temperature from API response".into())
    }
}