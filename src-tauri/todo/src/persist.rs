use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::Todo;

pub trait TodosDatabase {
    fn get_all_todos(&self) -> anyhow::Result<Vec<Todo>>;
    fn set_all_todos(&self, todos: Vec<Todo>) -> anyhow::Result<()>;
}

pub mod jsonfile {
    use super::{get_or_create_savefilename, TodosDatabase};

    use std::{
        fs::{File, OpenOptions},
        io::{BufReader, Write},
        path::{Path, PathBuf},
    };

    use anyhow::Context;
    use serde::{de::DeserializeOwned, Serialize};

    pub struct TodosJsonDB {
        filename: Option<PathBuf>,
    }

    impl Default for TodosJsonDB {
        fn default() -> Self {
            Self {
                filename: get_or_create_savefilename("todo.json"),
            }
        }
    }

    impl TodosJsonDB {
        fn get_filename(&self) -> anyhow::Result<&Path> {
            let name = self
                .filename
                .as_ref()
                .context("failed to setup a json file for the todos")?;

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

    pub fn read_json<Item: DeserializeOwned + Serialize>(filename: &Path) -> anyhow::Result<Item> {
        let p = &std::env::var("HOME")?;
        let file = open_file(&Path::new(p).join(filename))?;
        let reader = BufReader::new(&file);
        let item = serde_json::from_reader(reader)?;
        Ok(item)
    }

    pub fn write_json<Item: DeserializeOwned + Serialize>(
        filename: &Path,
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

pub mod binary {

    use anyhow::Context;
    use chrono::DateTime;

    use crate::TodoID;

    use super::*;

    impl Todo {
        fn to_binary(&self) -> Vec<u8> {
            let message_bin = self.message.as_bytes();

            let timestamp = self
                .created_at
                .0
                .timestamp_nanos_opt()
                .expect("failed to get timestamp nanos, not in range?");
            let time_bin = timestamp.to_be_bytes();
            let done_bin: u8 = if self.done { 1 } else { 0 };

            let data = [
                &self.message.len().to_be_bytes(), // first 8 bytes is message len
                message_bin,                       // next len bytes is message
                &time_bin,                         // next 8 bytes in timestamp
                &[done_bin],                       // last byte is 0 or 1 for isDone flag
            ]
            .concat();

            return data;
        }

        /// Expecting data to be a reverse byte buffer, so as to emulate a stack.
        fn from_binary(data: &mut Vec<u8>) -> anyhow::Result<Todo> {
            let mut message_len = [0u8; 8];
            for i in message_len.iter_mut() {
                *i = data.pop().context("empty data")?
            }

            let message_len = usize::from_be_bytes(message_len);

            let mut message = Vec::with_capacity(message_len);
            for _ in 0..message_len {
                let byte = data.pop().context("empty data")?;
                message.push(byte);
            }
            let message = String::from_utf8(message).context("message was not in utf-8")?;

            let mut timestamp_nanos = [0u8; 8];
            for i in timestamp_nanos.iter_mut() {
                *i = data.pop().context("empty data")?
            }

            let timestamp_nanos = i64::from_be_bytes(timestamp_nanos);
            let todo_time = crate::TodoTime(DateTime::from_timestamp_nanos(timestamp_nanos));

            let is_done_byte = data
                .pop()
                .context("empty data")
                .context("failed to read done byte")?;

            Ok(Self {
                id: TodoID::hash_message(&message),
                message,
                created_at: todo_time,
                done: is_done_byte != 0,
            })
        }
    }

    pub struct TodosBin {
        filename: Option<PathBuf>,
    }

    impl Default for TodosBin {
        fn default() -> Self {
            Self {
                filename: get_or_create_savefilename("todos.bin"),
            }
        }
    }

    impl TodosBin {
        fn get_filename(&self) -> anyhow::Result<&Path> {
            let name = self
                .filename
                .as_ref()
                .context("failed to setup a file for the todos")?;

            return Ok(name);
        }
    }

    impl TodosDatabase for TodosBin {
        fn get_all_todos(&self) -> anyhow::Result<Vec<Todo>> {
            let filename = self.get_filename()?;
            let mut data =
                std::fs::read(filename).context("failed to read binary save-file of todos")?;

            let mut todos = vec![];

            data.reverse(); // make it a stack.
            while !data.is_empty() {
                let t = Todo::from_binary(&mut data)?;
                todos.push(t)
            }

            Ok(todos)
        }

        fn set_all_todos(&self, todos: Vec<Todo>) -> anyhow::Result<()> {
            let filename = self.get_filename()?;
            let data = todos.iter().flat_map(|t| t.to_binary()).collect::<Vec<_>>();
            std::fs::write(filename, data).context("failed to write to todos binary save-file")?;
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        impl PartialEq for Todo {
            fn eq(&self, other: &Self) -> bool {
                if self.message != other.message {
                    return false;
                }
                if self.id != other.id {
                    return false;
                }
                if self.created_at != other.created_at {
                    return false;
                }
                if self.done != other.done {
                    return false;
                }

                return true;
            }
        }

        #[test]
        fn test_serde_binary() {
            let t = Todo::new("tesat".to_string());
            let mut data = t.to_binary();
            data.reverse();
            assert_eq!(t, Todo::from_binary(&mut data).unwrap());
            assert!(data.is_empty())
        }
    }
}

fn get_or_create_savefilename(name: &str) -> Option<PathBuf> {
    const DIR_NAME: &str = "mynd";

    let filename = Path::new(DIR_NAME).join(name);

    let get_dir_path = std::env::var("HOME")
        .context("failed to read $HOME var")
        .map(|path| -> PathBuf { Path::new(&path).join(DIR_NAME) });

    let filename = get_dir_path
        .and_then(|dir_path| {
            if !dir_path.is_dir() {
                return std::fs::create_dir(dir_path)
                    .context("failed to create a 'mynd' directory")
                    .map(|_| filename);
            }
            Ok(filename)
        })
        .map_err(|err| eprintln!("[ERROR] {err:#}"))
        .ok();

    return filename;
}
