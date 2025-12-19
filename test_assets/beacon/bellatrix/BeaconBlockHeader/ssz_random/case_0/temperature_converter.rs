
fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn celsius_to_kelvin(celsius: f64) -> f64 {
    celsius + 273.15
}

fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

fn fahrenheit_to_kelvin(fahrenheit: f64) -> f64 {
    celsius_to_kelvin(fahrenheit_to_celsius(fahrenheit))
}

fn kelvin_to_celsius(kelvin: f64) -> f64 {
    kelvin - 273.15
}

fn kelvin_to_fahrenheit(kelvin: f64) -> f64 {
    celsius_to_fahrenheit(kelvin_to_celsius(kelvin))
}

fn format_temperature(value: f64, unit: &str) -> String {
    format!("{:.2}°{}", value, unit)
}

fn main() {
    let celsius_temp = 25.0;
    
    println!("Temperature Conversions:");
    println!("Input: {}", format_temperature(celsius_temp, "C"));
    println!("Fahrenheit: {}", format_temperature(celsius_to_fahrenheit(celsius_temp), "F"));
    println!("Kelvin: {}", format_temperature(celsius_to_kelvin(celsius_temp), "K"));
    
    let fahrenheit_temp = 77.0;
    println!("\nFahrenheit {} to:", format_temperature(fahrenheit_temp, "F"));
    println!("Celsius: {}", format_temperature(fahrenheit_to_celsius(fahrenheit_temp), "C"));
    println!("Kelvin: {}", format_temperature(fahrenheit_to_kelvin(fahrenheit_temp), "K"));
    
    let kelvin_temp = 300.0;
    println!("\nKelvin {} to:", format_temperature(kelvin_temp, "K"));
    println!("Celsius: {}", format_temperature(kelvin_to_celsius(kelvin_temp), "C"));
    println!("Fahrenheit: {}", format_temperature(kelvin_to_fahrenheit(kelvin_temp), "F"));
}