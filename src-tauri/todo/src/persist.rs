use crate::Todo;

pub trait TodosDatabase {
    fn get_all_todos(&self) -> anyhow::Result<Vec<Todo>>;
    fn set_all_todos(&self, todos: Vec<Todo>) -> anyhow::Result<()>;
}

pub mod jsonfile {
    use super::TodosDatabase;

    use std::{
        fs::{File, OpenOptions},
        io::{BufReader, Write},
        path::{Path, PathBuf},
    };

    use anyhow::Context;
    use serde::{de::DeserializeOwned, Serialize};

    const FILE_NAME: &str = "mynd/todo.json";

    pub struct TodosJsonDB {
        filename: Option<String>,
    }

    impl Default for TodosJsonDB {
        fn default() -> Self {
            let filename = FILE_NAME.to_string();
            let filename = path("mynd")
                .and_then(|path| {
                    if !path.is_dir() {
                        return std::fs::create_dir(path)
                            .context("failed to create a 'mynd' directory")
                            .map(|_| filename);
                    }
                    Ok(filename)
                })
                .map_err(|err| eprintln!("[ERROR] {err:#}"))
                .ok();

            Self { filename }
        }
    }

    impl TodosJsonDB {
        fn get_filename(&self) -> anyhow::Result<&String> {
            let name = self
                .filename
                .as_ref()
                .context("failed to setup a file for the todos")?;

            return Ok(name);
        }
    }

    impl TodosDatabase for TodosJsonDB {
        fn get_all_todos(&self) -> anyhow::Result<Vec<crate::Todo>> {
            let json_file_name = self.get_filename()?;
            read_json(json_file_name)
        }

        fn set_all_todos(&self, todos: Vec<crate::Todo>) -> anyhow::Result<()> {
            let json_file_name = self.get_filename()?;
            write_json(json_file_name, todos)?;
            Ok(())
        }
    }

    pub fn path(name: &str) -> anyhow::Result<PathBuf> {
        let p = &std::env::var("HOME").context("failed to read $HOME var")?;
        Ok(Path::new(p).join(name))
    }

    pub fn read_json<Item: DeserializeOwned + Serialize>(filename: &str) -> anyhow::Result<Item> {
        let p = &std::env::var("HOME")?;
        let file = open_file(&Path::new(p).join(filename))?;
        let reader = BufReader::new(&file);
        let item = serde_json::from_reader(reader)?;
        Ok(item)
    }

    pub fn write_json<Item: DeserializeOwned + Serialize>(
        filename: &str,
        item: Item,
    ) -> anyhow::Result<()> {
        let json = serde_json::to_string::<Item>(&item)?;
        let p = &std::env::var("HOME")?;

        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .truncate(true)
            .open(Path::new(p).join(filename))?;

        write!(file, "{}", json)?;
        Ok(())
    }

    fn open_file(path: &Path) -> anyhow::Result<File> {
        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .truncate(false)
            .open(path)?;

        Ok(file)
    }
}
