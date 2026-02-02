use std::io;

fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn celsius_to_kelvin(celsius: f64) -> f64 {
    celsius + 273.15
}

fn main() {
    println!("Temperature Converter");
    println!("Enter temperature in Celsius:");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    let celsius: f64 = match input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Invalid input. Please enter a valid number.");
            return;
        }
    };

    let fahrenheit = celsius_to_fahrenheit(celsius);
    let kelvin = celsius_to_kelvin(celsius);

    println!("{:.2}°C = {:.2}°F", celsius, fahrenheit);
    println!("{:.2}°C = {:.2}K", celsius, kelvin);
}
use std::io;

fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn celsius_to_kelvin(celsius: f64) -> f64 {
    celsius + 273.15
}

fn main() {
    println!("Temperature Converter");
    println!("Enter temperature in Celsius:");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    let celsius: f64 = match input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Invalid input. Please enter a valid number.");
            return;
        }
    };

    let fahrenheit = celsius_to_fahrenheit(celsius);
    let kelvin = celsius_to_kelvin(celsius);

    println!("{:.2}°C = {:.2}°F", celsius, fahrenheit);
    println!("{:.2}°C = {:.2}K", celsius, kelvin);
}use std::io;

fn main() {
    println!("Temperature Converter");
    println!("Enter a temperature value followed by its unit (C or F), e.g., '25 C' or '77 F':");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.len() != 2 {
        eprintln!("Error: Please provide a value and a unit separated by a space.");
        return;
    }

    let value: f64 = match parts[0].parse() {
        Ok(num) => num,
        Err(_) => {
            eprintln!("Error: Invalid temperature value.");
            return;
        }
    };

    let unit = parts[1].to_uppercase();
    match unit.as_str() {
        "C" => {
            let fahrenheit = (value * 9.0 / 5.0) + 32.0;
            println!("{:.2}°C is {:.2}°F", value, fahrenheit);
        }
        "F" => {
            let celsius = (value - 32.0) * 5.0 / 9.0;
            println!("{:.2}°F is {:.2}°C", value, celsius);
        }
        _ => {
            eprintln!("Error: Invalid unit. Please use 'C' for Celsius or 'F' for Fahrenheit.");
        }
    }
}