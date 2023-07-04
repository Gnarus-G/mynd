use std::{error::Error, fs, sync::Mutex};

use cuid2::cuid;
use persist::{path, read_json, write_json};
use serde::{Deserialize, Serialize};

pub mod persist;

#[derive(Debug, Deserialize, Serialize, PartialEq, PartialOrd, Clone)]
pub struct TodoID(pub String);

impl Default for TodoID {
    fn default() -> Self {
        Self(cuid())
    }
}

impl From<String> for TodoID {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for TodoID {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, PartialOrd, Clone)]
pub struct TodoTime(chrono::DateTime<chrono::Utc>);

impl Default for TodoTime {
    fn default() -> Self {
        Self(chrono::Utc::now())
    }
}

impl TodoTime {
    pub fn now() -> Self {
        Self(chrono::Utc::now())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Todo {
    id: TodoID,
    pub message: String,
    created_at: TodoTime,
    pub done: bool,
}

impl Todo {
    pub fn new(message: String) -> Self {
        Self {
            id: Default::default(),
            message,
            created_at: Default::default(),
            done: false,
        }
    }
}

#[derive(Debug)]
pub struct Todos {
    list: Mutex<Vec<Todo>>,
}

impl Todos {
    pub fn new() -> Self {
        let t = Todos {
            list: Mutex::new(vec![]),
        };
        t.load();
        t
    }
    pub fn add(&self, message: &str) -> Result<(), Box<dyn Error>> {
        let mut list = self.list.lock().unwrap();
        let todo = Todo::new(message.to_string());
        list.insert(0, todo);

        write_json("mynd/todo.json", list.clone())?;
        Ok(())
    }

    fn find_index(&self, id: String) -> usize {
        self.list
            .lock()
            .unwrap()
            .iter()
            .enumerate()
            .find(|(_, t)| t.id == TodoID(id.clone()))
            .unwrap()
            .0
    }

    pub fn mark_done(&self, id: TodoID) {
        let idx = self.find_index(id.0);
        let mut list = self.list.lock().unwrap();
        let todo = list.get_mut(idx);

        if let Some(todo) = todo {
            todo.done = !todo.done;
        }

        write_json("mynd/todo.json", list.clone()).ok();
    }

    pub fn remove_done(&self) {
        let copy = self.get_all();
        *self.list.lock().unwrap() = copy.iter().filter(|t| !t.done).cloned().collect();
        write_json("mynd/todo.json", self.get_all()).ok();
    }

    pub fn move_up(&self, id: String) {
        let idx = self.find_index(id);

        if idx < self.list.lock().unwrap().len() {
            let curr = self.list.lock().unwrap()[idx].clone();
            let temp = self.list.lock().unwrap()[idx - 1].clone();

            self.list.lock().unwrap()[idx] = temp;
            self.list.lock().unwrap()[idx - 1] = curr;

            write_json("mynd/todo.json", self.get_all()).ok();
        }
    }

    pub fn move_down(&self, id: String) {
        let idx = self.find_index(id);

        if idx < self.list.lock().unwrap().len() {
            let curr = self.list.lock().unwrap()[idx].clone();
            let temp = self.list.lock().unwrap()[idx + 1].clone();

            self.list.lock().unwrap()[idx] = temp;
            self.list.lock().unwrap()[idx + 1] = curr;

            write_json("mynd/todo.json", self.get_all()).ok();
        }
    }

    pub fn load(&self) {
        let dir = path("mynd");

        if !dir.is_dir() {
            fs::create_dir(dir).expect("failed to create a 'mynd' directory");
        }

        let list = read_json("mynd/todo.json").unwrap_or_default();

        *self.list.lock().unwrap() = list;
    }

    pub fn get_all(&self) -> Vec<Todo> {
        self.list.lock().unwrap().clone()
    }
}
