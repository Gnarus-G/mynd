use std::error::Error;

use clap::Parser;
use mynd::{
    persist::{HasId, PersistenJson},
    TodoID,
};
use serde::{Deserialize, Serialize};

#[derive(Parser, Serialize, Deserialize, Debug)]
#[command(author, version, about)]
struct Reminder {
    #[arg(skip)]
    id: TodoID,
    /// What to remember.
    message: Option<String>,
    /// A message of how important this is to remember.
    #[arg(short, long, action = clap::ArgAction::Count)]
    priority: u8,
}

impl HasId for Reminder {
    fn id(&self) -> &str {
        &self.id.0
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let reminder = Reminder::parse();

    let mut tp = PersistenJson::new("reminders", "reminder")?;
    match reminder.message {
        Some(_) => tp.add(reminder)?,
        None => println!("{}", serde_json::to_string(&tp.items::<Reminder>()?)?),
    }

    Ok(())
}
