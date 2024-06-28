use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::{config::load_config, Todo};

pub trait TodosDatabase {
    fn get_all_todos(&self) -> anyhow::Result<Vec<Todo>>;
    fn set_all_todos(&self, todos: Vec<Todo>) -> anyhow::Result<()>;
}

#[derive(Debug)]
pub enum ActualTodosDB {
    JsonFile(jsonfile::TodosJsonDB),
    BinaryFile(binary::TodosBin),
}

impl Default for ActualTodosDB {
    fn default() -> Self {
        let cfg = load_config().unwrap_or_default();

        return match cfg.save_file_format {
            crate::config::SaveFileFormat::Json => {
                eprintln!("[INFO] using 'json' save file because of configuration.");
                Self::JsonFile(jsonfile::TodosJsonDB::default())
            }
            crate::config::SaveFileFormat::Binary => {
                eprintln!("[INFO] using 'binary' save file because of configuration.");
                Self::BinaryFile(binary::TodosBin::default())
            }
        };
    }
}

impl TodosDatabase for ActualTodosDB {
    fn get_all_todos(&self) -> anyhow::Result<Vec<Todo>> {
        match self {
            ActualTodosDB::JsonFile(db) => db.get_all_todos(),
            ActualTodosDB::BinaryFile(db) => db.get_all_todos(),
        }
    }

    fn set_all_todos(&self, todos: Vec<Todo>) -> anyhow::Result<()> {
        match self {
            ActualTodosDB::JsonFile(db) => db.set_all_todos(todos),
            ActualTodosDB::BinaryFile(db) => db.set_all_todos(todos),
        }
    }
}

pub mod jsonfile {
    use super::{get_or_create_savefilename, TodosDatabase};

    use std::{
        fs::{File, OpenOptions},
        io::{BufReader, Write},
        path::{Path, PathBuf},
    };

    use anyhow::{anyhow, Context};
    use serde::{de::DeserializeOwned, Serialize};

    #[derive(Debug)]
    pub struct TodosJsonDB {
        filename: anyhow::Result<PathBuf>,
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
            match &self.filename {
                Ok(p) => Ok(p),
                Err(err) => {
                    return Err(
                        anyhow!("{:#}", err).context("failed to setup a json file for the todos")
                    );
                }
            }
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
        let p =
            &std::env::var("HOME").context("failed to resolve the HOME environment variable")?;
        let file = open_file(&Path::new(p).join(filename))?;
        let reader = BufReader::new(&file);
        let item = serde_json::from_reader(reader).context("failed to read json data")?;
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
            .open(path)
            .context("failed to open json file for reading/writing")?;

        Ok(file)
    }
}

pub mod binary {

    use std::{fs::OpenOptions, io::Read};

    use anyhow::{anyhow, Context};
    use chrono::DateTime;

    use crate::TodoID;

    use super::*;

    fn into_int_bytes(int: usize) -> [u8; 4] {
        let int = int as u32;
        return int.to_be_bytes();
    }

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

            let version: &[u8] = &[1];
            let data = [
                version,                             // first byte is the version of this format
                &into_int_bytes(self.message.len()), // next 4 bytes is message len
                message_bin,                         // next len bytes is message
                &time_bin,                           // next 8 bytes in timestamp
                &[done_bin],                         // last byte is 0 or 1 for isDone flag
            ]
            .concat();

            return data;
        }

        /// Expecting data to be a reverse byte buffer, so as to emulate a stack.
        fn from_binary(data: &mut Vec<u8>) -> anyhow::Result<Todo> {
            let _version_byte = data.pop().context("empty data")?;

            let mut message_len = [0u8; 4];
            for i in message_len.iter_mut() {
                *i = data.pop().context("empty data")?
            }

            let message_len = u32::from_be_bytes(message_len);

            let mut message = Vec::with_capacity(message_len as usize);
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

    #[derive(Debug)]
    pub struct TodosBin {
        filename: anyhow::Result<PathBuf>,
    }

    impl Default for TodosBin {
        fn default() -> Self {
            Self {
                filename: get_or_create_savefilename("todo.bin"),
            }
        }
    }

    impl TodosBin {
        fn get_filename(&self) -> anyhow::Result<&Path> {
            match &self.filename {
                Ok(p) => Ok(p),
                Err(err) => {
                    return Err(
                        anyhow!("{:#}", err).context("failed to setup a binary file for the todos")
                    );
                }
            }
        }
    }

    impl TodosDatabase for TodosBin {
        fn get_all_todos(&self) -> anyhow::Result<Vec<Todo>> {
            let filename = self.get_filename()?;
            let mut file = OpenOptions::new()
                .read(true)
                .create(true)
                .append(true)
                .open(filename)?;

            let mut data = vec![];

            file.read_to_end(&mut data)
                .context("failed to read binary save-file of todos")?;

            get_todos_from_binary(&mut data)
        }

        fn set_all_todos(&self, todos: Vec<Todo>) -> anyhow::Result<()> {
            let filename = self.get_filename()?;
            let data = convert_todos_to_binary(&todos);
            std::fs::write(filename, data).context(anyhow!(
                "failed to write to todos binary save-file: {}",
                filename.display()
            ))?;
            Ok(())
        }
    }

    pub fn get_todos_from_binary(data: &mut Vec<u8>) -> anyhow::Result<Vec<Todo>> {
        if data.is_empty() {
            return Ok(vec![]);
        }

        let mut todos = Vec::with_capacity(data.len() / 14); // 14 is the intended minimun of bytes
                                                             // to represent a todo message, (i.e an
                                                             // empty message string)
        data.reverse(); // make it a stack.
        while !data.is_empty() {
            let t = Todo::from_binary(data)?;
            todos.push(t)
        }

        debug_assert!(!todos.is_empty());

        return Ok(todos);
    }

    fn convert_todos_to_binary(todos: &[Todo]) -> Vec<u8> {
        let data = todos.iter().flat_map(|t| t.to_binary()).collect::<Vec<_>>();
        return data;
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

        #[test]
        fn test_serde_binary_many() {
            let todos = [
                Todo::new("one".to_string()),
                Todo::new("two".to_string()),
                Todo::new("three".to_string()),
                Todo::new("adsfasd;lfkjasdf".to_string()),
            ];

            let mut data = convert_todos_to_binary(&todos);
            assert_eq!(todos.to_vec(), get_todos_from_binary(&mut data).unwrap());

            assert!(data.is_empty())
        }
    }
}

fn get_or_create_savefilename(filename: &str) -> anyhow::Result<PathBuf> {
    const DIR_NAME: &str = "mynd";

    let get_dir_path = std::env::var("HOME")
        .context("failed to read $HOME var")
        .map(|path| -> PathBuf { Path::new(&path).join(DIR_NAME) });

    let savefilepath = get_dir_path
        .and_then(|dir_path| {
            eprintln!(
                "[INFO] resolving mynd save directory as: {}",
                dir_path.display()
            );

            if !dir_path.is_dir() {
                return std::fs::create_dir(&dir_path)
                    .context("failed to create a 'mynd' directory")
                    .map(|_| dir_path);
            }

            Ok(dir_path)
        })
        .map(|path| path.join(filename));

    return savefilepath;
}
