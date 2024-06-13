use std::os::unix::process::CommandExt;

use anyhow::Context;
use clap::{Parser, Subcommand};
use colored::Colorize;
use todo::Todos;

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
    /// Delete a todo item, regardless of if it's done or not.
    Rm(remove::RemoveArgs),
    /// List all todos that aren't done.
    Ls {},

    /// Launch the GUI (mynd). Assuming it's in the path.
    Gui,

    /// Read and save todos from a given file
    Import(import::ImportArgs),

    /// Dump all todos as json.
    Dump {
        /// Only dump undone todo items
        #[arg(short = 't')]
        todo: bool,
    },

    /// Manage global configuration values.
    Config(manageconfigcli::ConfigArgs),
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let todos = Todos::load_up_with_persistor();

    match args.command {
        Some(c) => match c {
            Command::Done { ids } => {
                for id in ids {
                    todos.mark_done(id.clone())?;
                    eprintln!("[INFO] marked done todo id: {}", id);
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
            Command::Import(a) => a.handle()?,
            Command::Config(a) => a.handle()?,
            Command::Rm(a) => a.handle()?,
            Command::Gui => {
                let err = std::process::Command::new("mynd").exec();
                return Err(err).context("failed to run the executable `mynd`. See the README @ https://github.com/Gnarus-G/mynd");
            }
        },
        None => match args.message {
            Some(message) => {
                if message.is_empty() {
                    return Err(anyhow!("no sense in an empty todo message"));
                }
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

mod remove {
    use clap::Args;
    use todo::Todos;

    #[derive(Args, Debug)]
    pub struct RemoveArgs {
        /// Id(s) of the todo(s) to delete.
        ids: Vec<String>,
    }

    impl RemoveArgs {
        pub fn handle(self) -> anyhow::Result<()> {
            let todos = Todos::load_up_with_persistor();

            for id in self.ids {
                match todos.remove(id.clone()) {
                    Ok(_) => {
                        eprintln!("[INFO] deleted todo id: {}", id)
                    }
                    Err(err) => {
                        eprintln!("[ERROR] failed to remove todo id: {}", id);
                        eprintln!("[ERROR] {err:#}")
                    }
                }
            }

            Ok(())
        }
    }
}

mod import {
    use std::{ffi::OsStr, path::PathBuf};

    use anyhow::{anyhow, Context};
    use todo::persist::{binary, jsonfile, ActualTodosDB, TodosDatabase};

    use clap::Args;

    #[derive(Debug, Args)]
    pub struct ImportArgs {
        /// from which to read todo items
        file: PathBuf,
    }

    impl ImportArgs {
        pub fn handle(self) -> anyhow::Result<()> {
            let file = self.file;

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

            let imported_todos;

            match ext {
                    Ok("json") => {
                        imported_todos = jsonfile::read_json(&file)?;
                    }
                    Ok("bin") => {
                        let mut data =
                            std::fs::read(file).context("failed to read from import file")?;
                        imported_todos = binary::get_todos_from_binary(&mut data)?;
                    }
                    Err(err) => {
                        return Err(err.context("unsupported file extension"))
                    }
                    _ => unreachable!("unreachable assertion failed even though we are[should be] filter out unsupported extensions in an error"),
                }

            let mut todos = db
                .get_all_todos()
                .context("failed to load current set of todos")?;

            todos.extend(imported_todos);

            db.set_all_todos(todos)?;

            Ok(())
        }
    }
}

mod manageconfigcli {
    use std::io::stdout;

    use clap::{Args, Subcommand};

    use crate::config::{self, store_config};

    #[derive(Args, Debug)]
    pub struct ConfigProps {
        #[arg(short = 'f', long = "format")]
        /// The storage format of the collection of todo items.
        storage_format: config::SaveFileFormat,
    }

    #[derive(Subcommand, Debug)]
    pub enum ConfigActions {
        /// Update configuration values.
        Set(ConfigProps),
        /// Print configuration values to standard output as json.
        Show,
    }

    #[derive(Args, Debug)]
    pub struct ConfigArgs {
        #[command(subcommand)]
        command: ConfigActions,
    }

    impl ConfigArgs {
        pub fn handle(self) -> anyhow::Result<()> {
            match self.command {
                ConfigActions::Set(ConfigProps { storage_format }) => {
                    let cfg = config::MyndConfig {
                        save_file_format: storage_format,
                    };

                    store_config(cfg)?;
                }
                ConfigActions::Show => {
                    let cfg = config::load_config()?;
                    serde_json::to_writer_pretty(stdout(), &cfg)?;
                    println!()
                }
            };

            Ok(())
        }
    }
}
