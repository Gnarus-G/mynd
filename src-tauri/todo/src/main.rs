use std::error::Error;

use clap::Parser;
use todo::Todos;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// What to do.
    message: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    let tp = Todos::new();

    match args.message {
        Some(message) => {
            tp.add(&message)?;
        }
        None => {
            let todos = tp.get_all();
            println!("{}", serde_json::to_string(&todos)?);
        }
    }

    Ok(())
}
