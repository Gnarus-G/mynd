use std::error::Error;

use clap::{Parser, Subcommand};
use colored::Colorize;
use todo::Todos;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// What to do.
    message: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Mark one or more todo items as done.
    Done {
        /// Ids of the todo(s) to mark done.
        ids: Vec<String>,
    },
    /// List all todos that aren't done.
    Ls {},

    /// Dump all todos as json.
    Dump {
        /// Only dump undone todo items
        #[arg(short = 't')]
        todo: bool,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    let todos = Todos::new();

    match args.command {
        Some(c) => match c {
            Command::Done { ids } => {
                for id in ids {
                    todos.mark_done(id.into())
                }
            }
            Command::Ls {} => todos
                .get_all()
                .into_iter()
                .filter(|t| !t.done)
                .map(|t| t.message)
                .for_each(|m| {
                    println!("{}\n", m.yellow());
                }),
            Command::Dump { todo } => {
                let todos: Vec<_> = todos
                    .get_all()
                    .into_iter()
                    .filter(|t| !todo || !t.done)
                    .collect();

                println!("{}", serde_json::to_string(&todos)?);
            }
        },
        None => match args.message {
            Some(message) => {
                todos.add(&message)?;
            }
            None => {
                todos
                    .get_all()
                    .into_iter()
                    .map(|t| {
                        if t.done {
                            t.message.strikethrough().dimmed()
                        } else {
                            t.message.yellow()
                        }
                    })
                    .for_each(|m| {
                        println!("{}", m);
                    });
            }
        },
    }

    Ok(())
}
