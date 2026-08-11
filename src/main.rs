use std::env;

use aetherion::{Command, execute};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    match Command::parse(&args).and_then(execute) {
        Ok(Some(output)) => println!("{output}"),
        Ok(None) => {}
        Err(error) => {
            if let Some(json) = error.json {
                println!("{json}");
            }
            eprintln!("aetherion: {}", error.message);
            std::process::exit(error.exit_code);
        }
    }
}
