use std::error::Error;

use clap::Parser;
use mynd::{
    persist::{HasId, PersistenJson},
    TodoID, TodoTime,
};
use serde::{Deserialize, Serialize};

#[derive(Parser, Serialize, Deserialize, Debug)]
#[command(author, version, about)]
struct Todo {
    #[arg(skip)]
    id: TodoID,
    /// What to do.
    message: Option<String>,
    /// A measure of how much of a drag this will be.
    #[arg(short, long, action = clap::ArgAction::Count)]
    drag: u8,
    /// A message of how important this is to do.
    #[arg(short, long, action = clap::ArgAction::Count)]
    priority: u8,

    #[arg(skip)]
    created_at: TodoTime,
}

impl HasId for Todo {
    fn id(&self) -> &str {
        &self.id.0
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let todo = Todo::parse();

    let mut tp = PersistenJson::new("todos", "todo")?;

    match todo.message {
        Some(_) => tp.add(todo)?,
        None => {
            let mut todos = tp.items::<Todo>()?;

            todos.sort_by(|a, b| {
                a.created_at
                    .partial_cmp(&b.created_at)
                    .expect("comparison is possible")
            });

            println!("{}", serde_json::to_string(&todos)?);
        }
    }

    Ok(())
}
