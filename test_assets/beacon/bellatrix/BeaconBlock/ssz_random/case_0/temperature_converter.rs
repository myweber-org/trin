use std::io;

fn main() {
    loop {
        println!("Temperature Converter");
        println!("1. Celsius to Fahrenheit");
        println!("2. Fahrenheit to Celsius");
        println!("3. Exit");

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read line");
        let choice: u32 = match choice.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Please enter a valid number!");
                continue;
            }
        };

        match choice {
            1 => {
                let celsius = get_temperature_input("Enter temperature in Celsius: ");
                let fahrenheit = celsius_to_fahrenheit(celsius);
                println!("{:.2}°C is equal to {:.2}°F", celsius, fahrenheit);
            }
            2 => {
                let fahrenheit = get_temperature_input("Enter temperature in Fahrenheit: ");
                let celsius = fahrenheit_to_celsius(fahrenheit);
                println!("{:.2}°F is equal to {:.2}°C", fahrenheit, celsius);
            }
            3 => {
                println!("Goodbye!");
                break;
            }
            _ => {
                println!("Invalid choice. Please select 1, 2, or 3.");
            }
        }
        println!();
    }
}

fn get_temperature_input(prompt: &str) -> f64 {
    loop {
        println!("{}", prompt);
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        match input.trim().parse() {
            Ok(num) => return num,
            Err(_) => {
                println!("Please enter a valid number!");
                continue;
            }
        }
    }
}

fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}