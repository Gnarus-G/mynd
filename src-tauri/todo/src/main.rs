use std::path::PathBuf;

use anyhow::anyhow;
use clap::{Parser, Subcommand};
use colored::Colorize;
use todo::{
    persist::{
        jsonfile::{self},
        ActualTodosDB, TodosDatabase,
    },
    Todos,
};

mod config;

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

    /// Read and save todos from a given file
    Import {
        /// from which to read todo items
        file: PathBuf,
    },

    /// Dump all todos as json.
    Dump {
        /// Only dump undone todo items
        #[arg(short = 't')]
        todo: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let todos = Todos::load_up_with_persistor();

    match args.command {
        Some(c) => match c {
            Command::Done { ids } => {
                for id in ids {
                    todos.mark_done(id.into())?
                }
            }
            Command::Ls {} => todos
                .get_all()?
                .into_iter()
                .filter(|t| !t.done)
                .map(|t| t.message)
                .for_each(|m| {
                    println!("{}\n", m.yellow());
                }),
            Command::Dump { todo } => {
                let todos: Vec<_> = todos
                    .get_all()?
                    .into_iter()
                    .filter(|t| !todo || !t.done)
                    .collect();

                println!("{}", serde_json::to_string(&todos)?);
            }
            Command::Import { file } => {
                let ext = file
                    .extension()
                    .filter(|ext| *ext == "json")
                    .and_then(|ext| ext.to_str());

                match ext {
                    Some("json") => {
                        let todos = jsonfile::read_json(&file)?;
                        let db = ActualTodosDB::default();

                        db.set_all_todos(todos)?
                    }
                    _ => {
                        return Err(anyhow!("unsupported file extension received")
                            .context("extension is not one of the only supported: `json`"));
                    }
                }
            }
        },
        None => match args.message {
            Some(message) => {
                todos.add(&message)?;
            }
            None => {
                todos
                    .get_all()?
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
