use std::fs;
use std::io::{self, Write};
use std::path::Path;

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input.trim().to_string()
}

fn write_dotenv(file: &Path) {
    println!("Enter your PostgreSQL connection details:");

    let user = prompt("User: ");
    let password = prompt("Password: ");
    let host = prompt(
        "Host (default: `localhost`, use `@172.17.0.1/@host.docker.internal` if you work with Docker on localhost.): ",
    );
    let host = if host.is_empty() {
        "localhost".to_string()
    } else {
        host
    };
    let port = prompt("Port (default: 5432): ");
    let port = if port.is_empty() {
        "5432".to_string()
    } else {
        port
    };
    let database = prompt("Database name: ");

    println!("Enter your bot token:");
    let bot_token = prompt("> ");

    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        user, password, host, port, database
    );

    fs::write(
        file,
        format!("DB_USER={user}\nDB_PASSWORD={password}\nDB_NAME={database}\n\nDATABASE_URL={}\nBOT_TOKEN={}", database_url, bot_token),
    )
    .expect("Failed to write .env file");

    println!("`.env` file written successfully!");
}

fn handle_input(loop_msg: &str, default_option: char) -> char {
    print!("{}", loop_msg);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read user input.");

    match input.to_lowercase().trim() {
        "y" => 'y',
        "n" => 'n',
        _ => default_option,
    }
}

fn main() {
    let dotenv = Path::new(".env");
    println!("SELECTED FILE: {dotenv:?}\n");

    loop {
        if !dotenv.exists() {
            println!("{dotenv:?} file doesn't exist.");
            let message = "Do you want to create it? (Y/n): ";
            let input: char = handle_input(message, 'y');
            match input {
                'n' => {
                    println!("Alright, exiting.");
                    break;
                }
                'y' => {
                    write_dotenv(dotenv);
                    break;
                }
                _ => unreachable!(),
            }
        } else {
            println!("{dotenv:?} file already exists.");
            let message: &str = "Do you want to see the file? (y/N): ";
            let input: char = handle_input(message, 'n');
            if input == 'y' {
                if let Ok(content) = fs::read_to_string(dotenv) {
                    println!("File content:\n{}", content);
                } else {
                    eprintln!("Failed to read file content!");
                }
            }

            let delete_message = "Do you want to delete the file? (y/N): ";
            let del_option = handle_input(delete_message, 'n');
            if del_option == 'y' {
                let _ = fs::remove_file(dotenv);
                println!("File deleted.");
            } else {
                break;
            }
        }
    }
}
