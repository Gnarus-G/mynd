// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use todo::{persist::jsonfile::TodosJsonDB, Todo, TodoID, Todos};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn load(todos: tauri::State<'_, TodosState>) -> Vec<Todo> {
    todos.reload().expect("failed to relaod todos");
    todos.get_all().expect("failed to fetch all todos")
}

#[tauri::command]
fn add(todo: String, todos: tauri::State<'_, TodosState>) -> TodosCommandResult {
    todos.add(&todo).into_command_result()?;
    todos.get_all().into_command_result()
}

#[tauri::command]
fn remove(id: String, todos: tauri::State<'_, TodosState>) -> TodosCommandResult {
    todos
        .mark_done(TodoID(id))
        .expect("failed to remove (mark done) a todo");
    todos.get_all().into_command_result()
}

#[tauri::command]
fn remove_done(todos: tauri::State<'_, TodosState>) -> TodosCommandResult {
    todos.remove_done().expect("failed to remove done todos");
    todos.get_all().into_command_result()
}

#[tauri::command]
fn move_up(id: String, todos: tauri::State<'_, TodosState>) -> TodosCommandResult {
    todos.move_up(id).expect("failed to move a todo up");
    todos.get_all().into_command_result()
}

#[tauri::command]
fn move_down(id: String, todos: tauri::State<'_, TodosState>) -> TodosCommandResult {
    todos.move_down(id).expect("failed to move a todo down");
    todos.get_all().into_command_result()
}

#[tauri::command]
fn move_below(
    id: String,
    target_id: String,
    todos: tauri::State<'_, TodosState>,
) -> TodosCommandResult {
    todos
        .move_below(id, target_id)
        .expect("failed to move a todo below another");
    todos.get_all().into_command_result()
}

fn main() {
    tauri::Builder::default()
        .manage(initial_todos_state())
        .invoke_handler(tauri::generate_handler![
            load,
            add,
            remove,
            move_up,
            move_down,
            remove_done,
            move_below
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

type TodosState = Todos<TodosJsonDB>;

fn initial_todos_state() -> TodosState {
    Todos::load_up_with_persistor()
}

type CommandResult<T> = Result<T, String>;

trait MapToCommandResult<T> {
    fn into_command_result(self) -> CommandResult<T>;
}

impl<T> MapToCommandResult<T> for anyhow::Result<T> {
    fn into_command_result(self) -> CommandResult<T> {
        self.map_err(|err| err.to_string())
    }
}

type TodosCommandResult = CommandResult<Vec<Todo>>;
