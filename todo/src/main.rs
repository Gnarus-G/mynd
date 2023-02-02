use std::error::Error;

use clap::{Parser, Subcommand};
use mynd::{
    persist::{HasId, PersistenJson},
    TodoID, TodoTime,
};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// What to do.
    message: Option<String>,
    /// A measure of how much of a drag this will be.
    #[arg(short, long, action = clap::ArgAction::Count)]
    drag: u8,
    /// A message of how important this is to do.
    #[arg(short, long, action = clap::ArgAction::Count)]
    priority: u8,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Delete one or more todo items.
    Rm {
        /// Ids of the todo(s) to delete.
        ids: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct Todo {
    id: TodoID,
    message: String,
    drag: u8,
    priority: u8,
    created_at: TodoTime,
}

impl HasId for Todo {
    fn id(&self) -> &str {
        &self.id.0
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    let mut tp = PersistenJson::new("todos", "todo")?;

    match args.command {
        Some(Command::Rm { ids }) => tp.remove_all(&ids),
        None => match args.message {
            Some(message) => {
                tp.add(Todo {
                    id: Default::default(),
                    message,
                    drag: args.drag,
                    priority: args.priority,
                    created_at: Default::default(),
                })?;
            }
            None => {
                let mut todos = tp.items::<Todo>()?;

                todos.sort_by(|a, b| {
                    a.created_at
                        .partial_cmp(&b.created_at)
                        .expect("comparison is possible")
                });

                println!("{}", serde_json::to_string(&todos)?);
            }
        },
    }

    Ok(())
}
