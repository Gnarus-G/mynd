mod persist;

use std::error::Error;

use clap::{Args, Parser, Subcommand};
use persist::PersistenJson;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Create, or list, an item to to do later.
    Todo(Todo),
    Remind(Reminder),
}

#[derive(Args, Serialize, Deserialize, Debug)]
struct Todo {
    /// What to do.
    message: Option<String>,
    /// A measure of how much of a drag this will be.
    #[arg(short, long, action = clap::ArgAction::Count)]
    drag: u8,
    /// A message of how important this is to do.
    #[arg(short, long, action = clap::ArgAction::Count)]
    priority: u8,
}

#[derive(Args, Serialize, Deserialize, Debug)]
struct Reminder {
    /// What to remember.
    message: Option<String>,
    /// A message of how important this is to remember.
    #[arg(short, long, action = clap::ArgAction::Count)]
    priority: u8,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    match args.command {
        Command::Todo(todo) => {
            let mut tp = PersistenJson::new("todos", "todo")?;
            match todo.message {
                Some(_) => tp.add(todo)?,
                None => println!("{}", serde_json::to_string(&tp.items::<Todo>()?)?),
            }
        }
        Command::Remind(reminder) => {
            let mut tp = PersistenJson::new("reminders", "reminder")?;
            match reminder.message {
                Some(_) => tp.add(reminder)?,
                None => println!("{}", serde_json::to_string(&tp.items::<Reminder>()?)?),
            }
        }
    }

    Ok(())
}
