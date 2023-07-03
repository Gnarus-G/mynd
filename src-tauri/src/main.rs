// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, fs, path::Path, sync::Mutex};

use mynd::{
    persist::{path, read_json, write_json},
    TodoID, TodoTime,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Todo {
    id: TodoID,
    message: String,
    created_at: TodoTime,
}

impl Todo {
    pub fn new(message: String) -> Self {
        Self {
            id: Default::default(),
            message,
            created_at: Default::default(),
        }
    }
}

#[derive(Debug)]
struct Todos {
    list: Mutex<Vec<Todo>>,
}

impl Todos {
    pub fn new() -> Self {
        let dir = path("mynd");

        if !dir.is_dir() {
            fs::create_dir(dir).expect("failed to create a 'mynd' directory");
        }

        let list = read_json("mynd/todo.json").unwrap_or_default();

        Todos {
            list: Mutex::new(list),
        }
    }
    pub fn add(&self, message: &str) -> Result<(), Box<dyn Error>> {
        let mut g = self.list.lock().unwrap();
        let todo = Todo::new(message.to_string());
        g.insert(0, todo);

        write_json("mynd/todo.json", g.clone())?;
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

    pub fn remove(&self, id: TodoID) {
        let idx = self.find_index(id.0);
        self.list.lock().unwrap().remove(idx);

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

    pub fn get_all(&self) -> Vec<Todo> {
        self.list.lock().unwrap().clone()
    }
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn load(state: tauri::State<'_, Todos>) -> Vec<Todo> {
    state.get_all()
}

#[tauri::command]
fn add(todo: String, todos: tauri::State<'_, Todos>) -> Result<Vec<Todo>, String> {
    todos.add(&todo).map_err(|e| e.to_string())?;
    Ok(todos.get_all())
}

#[tauri::command]
fn remove(id: String, state: tauri::State<'_, Todos>) -> Vec<Todo> {
    state.remove(TodoID(id));
    state.get_all()
}

#[tauri::command]
fn move_up(id: String, todos: tauri::State<'_, Todos>) -> Vec<Todo> {
    todos.move_up(id);
    todos.get_all()
}

#[tauri::command]
fn move_down(id: String, todos: tauri::State<'_, Todos>) -> Vec<Todo> {
    todos.move_down(id);
    todos.get_all()
}

fn main() {
    tauri::Builder::default()
        .manage(Todos::new())
        .invoke_handler(tauri::generate_handler![
            load, add, remove, move_up, move_down
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
