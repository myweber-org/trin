
fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn main() {
    let celsius_temps = [0.0, 20.0, 37.0, 100.0];
    
    for &temp in &celsius_temps {
        let fahrenheit = celsius_to_fahrenheit(temp);
        println!("{:.1}°C = {:.1}°F", temp, fahrenheit);
    }
}fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn main() {
    let celsius_temp = 25.0;
    let fahrenheit_temp = celsius_to_fahrenheit(celsius_temp);
    println!("{}°C is equal to {}°F", celsius_temp, fahrenheit_temp);
}use std::io;

fn main() {
    println!("Temperature Converter");
    println!("1. Celsius to Fahrenheit");
    println!("2. Fahrenheit to Celsius");

    let choice: u32 = loop {
        println!("Please enter your choice (1 or 2):");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        match input.trim().parse() {
            Ok(num) if num == 1 || num == 2 => break num,
            _ => println!("Invalid input. Please enter 1 or 2."),
        }
    };

    let temperature: f64 = loop {
        println!("Enter the temperature to convert:");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        match input.trim().parse() {
            Ok(num) => break num,
            Err(_) => println!("Invalid input. Please enter a number."),
        }
    };

    let converted = if choice == 1 {
        celsius_to_fahrenheit(temperature)
    } else {
        fahrenheit_to_celsius(temperature)
    };

    let (from_unit, to_unit) = if choice == 1 {
        ("°C", "°F")
    } else {
        ("°F", "°C")
    };

    println!("{:.2}{} is equal to {:.2}{}", temperature, from_unit, converted, to_unit);
}

fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}