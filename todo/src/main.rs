use std::error::Error;

use clap::Parser;
use mynd::persist::PersistenJson;
use serde::{Deserialize, Serialize};

#[derive(Parser, Serialize, Deserialize, Debug)]
#[command(author, version, about)]
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

fn main() -> Result<(), Box<dyn Error>> {
    let todo = Todo::parse();

    let mut tp = PersistenJson::new("todos", "todo")?;

    match todo.message {
        Some(_) => tp.add(todo)?,
        None => println!("{}", serde_json::to_string(&tp.items::<Todo>()?)?),
    }

    Ok(())
}
