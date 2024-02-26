// File is purposefully not included in the crates
// It only serves as a small script to generate your dotenv file.

use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

fn write_dotenv(file: &Path) {
    println!("*Enter the bot token*");
    io::stdout().flush().unwrap();

    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read user input.");

    fs::write(file, format!("BOT_TOKEN={}", input)).expect("Error writing to the file.");
}

fn handle_input(loop_msg: &str, default_option: char) -> char {
    print!("{}", loop_msg);
    io::stdout().flush().unwrap();

    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read user input.");

    let result = match input.to_lowercase().trim().parse() {
        Ok(result) if result == 'y' || result == 'n' => result,
        _ => default_option,
    };

    result
}

fn main() {
    let dotenv = Path::new(".env");
    println!("SELECTED FILE: {dotenv:?}\n");
    loop {
        if !dotenv.exists() {
            println!("{dotenv:?} file doesn't exist.");
            let message = "Do you want to create it? (Y/n): ";
            let input: char = handle_input(&message, 'y');
            match input {
                'n' => 'block: {
                    println!("Alright...");
                    break 'block;
                }
                'y' => 'block: {
                    write_dotenv(&dotenv);
                    break 'block;
                }
                _ => unreachable!(),
            }
            return;
        }
        println!("{dotenv:?} file already exists.");
        let message: &str = "Do you want to see the file? (y/N): ";
        let input: char = handle_input(&message, 'n');
        if input == 'n' {
            println!("Alright...");
            return;
        }
        if let Ok(content) = fs::read_to_string(dotenv) {
            println!("File content:\n{}", content);
        } else {
            eprintln!("Failed to read file content!");
        }
        let delete_message = "Do you want to delete the file? (y/N): ";
        let del_option = handle_input(&delete_message, 'n');
        let _ = match del_option {
            'y' => fs::remove_file(dotenv),
            'n' => break,
            _ => unreachable!(),
        };
    }
}
