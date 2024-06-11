use std::{ffi::OsStr, path::PathBuf};

use anyhow::{anyhow, Context};
use clap::{Parser, Subcommand};
use colored::Colorize;
use todo::{
    persist::{binary, jsonfile, ActualTodosDB, TodosDatabase},
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
                let supported_extensions = &["json", "bin"].map(OsStr::new);

                let ext = file
                    .extension()
                    .filter(|ext| supported_extensions.contains(ext))
                    .context(anyhow!(
                        "extension is not one of the only supported: {:?}",
                        supported_extensions.map(|s| s.to_string_lossy()),
                    ))
                    .and_then(|e| e.to_str().context("file extension is not in utf-8"));

                let db = ActualTodosDB::default();

                match ext {
                    Ok("json") => {
                        let todos = jsonfile::read_json(&file)?;
                        db.set_all_todos(todos)?
                    }
                    Ok("bin") => {
                        let mut data =
                            std::fs::read(file).context("failed to read from import file")?;
                        let todos = binary::get_todos_from_binary(&mut data)?;
                        db.set_all_todos(todos)?;
                    }
                    Err(err) => {
                        return Err(err.context("unsupported file extension"))
                    }
                    _ => unreachable!("unreachable assertion failed even though we are[should be] filter out unsupported extensions in an error"),
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
