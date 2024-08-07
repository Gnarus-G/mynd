use anyhow::Context;
use todo::{persist::ActualTodosDB, Todo, Todos};

type TodosState = Todos<ActualTodosDB>;

fn initial_todos_state() -> TodosState {
    Todos::load_up_with_persistor()
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn load(todos: tauri::State<'_, TodosState>) -> TodosCommandResult {
    todos
        .reload()
        .context("failed to reload todos")
        .into_command_result()?;

    todos
        .get_all()
        .context("failed to fetch all todos")
        .into_command_result()
}

#[tauri::command]
fn add(todo: String, todos: tauri::State<'_, TodosState>) -> TodosCommandResult {
    todos.add_message(&todo).into_command_result()?;
    todos.flush().into_command_result()
}

#[tauri::command]
fn remove(id: String, todos: tauri::State<'_, TodosState>) -> TodosCommandResult {
    todos
        .mark_done(&id)
        .context("failed to remove (mark done) a todo")
        .into_command_result()?;

    todos.flush().into_command_result()
}

#[tauri::command]
fn delete(id: String, todos: tauri::State<'_, TodosState>) -> TodosCommandResult {
    todos
        .remove(&id)
        .context("failed to remove a todo")
        .into_command_result()?;

    todos.flush().into_command_result()
}

#[tauri::command]
fn remove_done(todos: tauri::State<'_, TodosState>) -> TodosCommandResult {
    todos
        .remove_done()
        .context("failed to remove done todos")
        .into_command_result()?;

    todos.get_all().into_command_result()
}

#[tauri::command]
fn move_up(id: String, todos: tauri::State<'_, TodosState>) -> TodosCommandResult {
    todos
        .move_up(id)
        .context("failed to move a todo up")
        .into_command_result()?;

    todos.get_all().into_command_result()
}

#[tauri::command]
fn move_down(id: String, todos: tauri::State<'_, TodosState>) -> TodosCommandResult {
    todos
        .move_down(id)
        .context("failed to move a todo down")
        .into_command_result()?;

    todos.get_all().into_command_result()
}

#[tauri::command]
fn move_below(
    id: String,
    target_id: String,
    todos: tauri::State<'_, TodosState>,
) -> TodosCommandResult {
    todos
        .move_below(&id, &target_id)
        .context("failed to move a todo below another")
        .into_command_result()?;

    todos.get_all().into_command_result()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(initial_todos_state())
        .invoke_handler(tauri::generate_handler![
            load,
            add,
            remove,
            delete,
            move_up,
            move_down,
            remove_done,
            move_below
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

type CommandResult<T> = Result<T, String>;

trait MapToCommandResult<T> {
    fn into_command_result(self) -> CommandResult<T>;
}

impl<T> MapToCommandResult<T> for anyhow::Result<T> {
    fn into_command_result(self) -> CommandResult<T> {
        self.map_err(|err| format!("{:#}", err))
    }
}

type TodosCommandResult = CommandResult<Vec<Todo>>;
