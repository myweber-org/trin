use std::io;

fn main() {
    println!("Temperature Converter (Celsius to Fahrenheit)");

    loop {
        println!("\nEnter temperature in Celsius (or 'q' to quit):");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let input = input.trim();
        if input.eq_ignore_ascii_case("q") {
            println!("Goodbye!");
            break;
        }

        match input.parse::<f64>() {
            Ok(celsius) => {
                let fahrenheit = celsius * 1.8 + 32.0;
                println!("{:.2}°C = {:.2}°F", celsius, fahrenheit);
            }
            Err(_) => {
                println!("Please enter a valid number.");
            }
        }
    }
}