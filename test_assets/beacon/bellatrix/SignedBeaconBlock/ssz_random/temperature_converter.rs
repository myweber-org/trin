
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

fn main() {
    let celsius_temp = 25.0;
    println!("{:.2}°C = {:.2}°F", celsius_temp, celsius_to_fahrenheit(celsius_temp));
    println!("{:.2}°C = {:.2}K", celsius_temp, celsius_to_kelvin(celsius_temp));
    
    let fahrenheit_temp = 77.0;
    println!("{:.2}°F = {:.2}°C", fahrenheit_temp, fahrenheit_to_celsius(fahrenheit_temp));
    println!("{:.2}°F = {:.2}K", fahrenheit_temp, fahrenheit_to_kelvin(fahrenheit_temp));
    
    let kelvin_temp = 298.15;
    println!("{:.2}K = {:.2}°C", kelvin_temp, kelvin_to_celsius(kelvin_temp));
    println!("{:.2}K = {:.2}°F", kelvin_temp, kelvin_to_fahrenheit(kelvin_temp));
}use std::io;

fn main() {
    println!("Temperature Converter (Celsius to Fahrenheit)");
    println!("Enter temperature in Celsius:");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let celsius: f64 = match input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Please enter a valid number!");
            return;
        }
    };

    let fahrenheit = celsius * 1.8 + 32.0;
    println!("{:.2}°C is equal to {:.2}°F", celsius, fahrenheit);
}