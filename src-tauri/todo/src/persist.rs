use crate::Todo;

pub trait TodosDatabase {
    fn get_all_todos(&self) -> Vec<Todo>;
    fn set_all_todos(&self, todos: Vec<Todo>);
}

pub mod jsonfile {
    use super::TodosDatabase;

    use std::{
        error::Error,
        fs::{File, OpenOptions},
        io::{BufReader, Write},
        path::{Path, PathBuf},
    };

    use serde::{de::DeserializeOwned, Serialize};

    pub struct TodosJsonDB {
        filename: String,
    }

    impl Default for TodosJsonDB {
        fn default() -> Self {
            let dir = path("mynd");

            if !dir.is_dir() {
                std::fs::create_dir(dir).expect("failed to create a 'mynd' directory");
            }

            Self {
                filename: "mynd/todo.json".to_string(),
            }
        }
    }

    impl TodosDatabase for TodosJsonDB {
        fn get_all_todos(&self) -> Vec<crate::Todo> {
            read_json(&self.filename).unwrap_or_default()
        }

        fn set_all_todos(&self, todos: Vec<crate::Todo>) {
            write_json(&self.filename, todos).ok();
        }
    }

    pub fn path(name: &str) -> PathBuf {
        let p = &std::env::var("HOME").expect("failed to read $HOME var");
        Path::new(p).join(name)
    }

    pub fn read_json<Item: DeserializeOwned + Serialize>(
        filename: &str,
    ) -> Result<Item, Box<dyn Error>> {
        let p = &std::env::var("HOME")?;
        let file = open_file(&Path::new(p).join(filename))?;
        let reader = BufReader::new(&file);
        let item = serde_json::from_reader(reader)?;
        Ok(item)
    }

    pub fn write_json<Item: DeserializeOwned + Serialize>(
        filename: &str,
        item: Item,
    ) -> Result<(), Box<dyn Error>> {
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

    fn open_file(path: &Path) -> Result<File, Box<dyn Error>> {
        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .truncate(false)
            .open(path)?;

        Ok(file)
    }
}
