// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use todo::{Todo, TodoID, Todos};

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
fn remove(id: String, todos: tauri::State<'_, Todos>) -> Vec<Todo> {
    todos.mark_done(TodoID(id));
    todos.get_all()
}

#[tauri::command]
fn remove_done(todos: tauri::State<'_, Todos>) -> Vec<Todo> {
    todos.remove_done();
    todos.get_all()
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
            load,
            add,
            remove,
            move_up,
            move_down,
            remove_done
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
