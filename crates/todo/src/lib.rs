use std::{
    fmt::Display,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::Duration,
};

use anyhow::anyhow;
use chrono::{Local, TimeZone};
use collection::array::TodoArrayList;
use collection::TodoCollection;
use persist::{ActualTodosDB, TodosDatabase};
use serde::{Deserialize, Serialize};

mod collection;
pub mod config;
pub mod persist;

pub fn open_web_app() -> anyhow::Result<()> {
    if launch_installed_web_app() {
        return Ok(());
    }

    let url = config::web_url()?;
    open::that(&url).map_err(anyhow::Error::from)?;
    Ok(())
}

fn launch_installed_web_app() -> bool {
    let Some(applications_dir) = applications_dir() else {
        return false;
    };
    let Some(desktop_file) = find_installed_web_app(&applications_dir) else {
        return false;
    };

    let desktop_id = desktop_file
        .file_stem()
        .map(|name| name.to_string_lossy().into_owned());
    if let Some(desktop_id) = desktop_id {
        if start_command(Command::new("gtk-launch").arg(desktop_id)) {
            return true;
        }
    }

    start_command(Command::new("gio").arg("launch").arg(desktop_file))
}

fn start_command(command: &mut Command) -> bool {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command_started(command.spawn())
}

fn command_started(child: std::io::Result<Child>) -> bool {
    let Ok(mut child) = child else {
        return false;
    };
    thread::sleep(Duration::from_millis(150));
    match child.try_wait() {
        Ok(Some(status)) => status.success(),
        Ok(None) => true,
        Err(_) => false,
    }
}

fn applications_dir() -> Option<PathBuf> {
    if let Some(data_home) = std::env::var_os("XDG_DATA_HOME") {
        return Some(PathBuf::from(data_home).join("applications"));
    }
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share/applications"))
}

fn find_installed_web_app(applications_dir: &Path) -> Option<PathBuf> {
    std::fs::read_dir(applications_dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "desktop")
        })
        .find(|path| {
            std::fs::read_to_string(path).is_ok_and(|desktop_entry| {
                desktop_entry.lines().any(|line| line == "Name=Mynd")
                    && desktop_entry
                        .lines()
                        .any(|line| line.starts_with("Exec=") && line.contains("--app-id="))
            })
        })
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Clone, Hash)]
pub struct TodoID(pub Box<str>);
impl TodoID {
    pub fn hash_message(message: &str) -> TodoID {
        TodoID(sha256::digest(message).into())
    }
}

impl From<String> for TodoID {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}

impl From<&str> for TodoID {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, PartialOrd, Clone)]
pub struct TodoTime(chrono::DateTime<chrono::Utc>);

impl TodoTime {
    pub fn to_local_date_string(&self) -> String {
        Local
            .from_utc_datetime(&self.0.naive_utc())
            .format("%m/%d/%Y %H:%M")
            .to_string()
    }
}

impl Display for TodoTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
    pub id: TodoID,
    pub message: String,
    pub created_at: TodoTime,
    pub done: bool,
}

impl Todo {
    pub fn new(message: String) -> Self {
        Self {
            id: TodoID::hash_message(&message),
            message,
            created_at: Default::default(),
            done: false,
        }
    }
}

#[derive(Debug)]
pub struct Todos<DB: TodosDatabase> {
    pub db: DB,
}

impl<DB: TodosDatabase> Todos<DB> {
    pub fn new(db: DB) -> Self {
        Self { db }
    }
}

impl Todos<ActualTodosDB> {
    pub fn load_up_with_persistor() -> Todos<ActualTodosDB> {
        let db = ActualTodosDB::default();
        Todos { db }
    }
}

impl<DB: TodosDatabase> Todos<DB> {
    pub fn reload(&self) -> anyhow::Result<()> {
        eprintln!("[TRACE] reloading todos");
        self.db.get_all_todos()?;
        Ok(())
    }

    fn mutate<R>(
        &self,
        operation: impl FnOnce(&mut TodoArrayList) -> anyhow::Result<R>,
    ) -> anyhow::Result<(R, Vec<Todo>)> {
        let mut operation = Some(operation);
        let mut result = None;
        let todos = self.db.update_todos(|todos| {
            let mut list = TodoArrayList::from(todos);
            result =
                Some(operation.take().expect(
                    "database update called the operation more than once",
                )(&mut list)?);
            Ok(list.get_all())
        })?;

        Ok((
            result.expect("database update did not call the operation"),
            todos,
        ))
    }

    pub fn add_message(&self, message: &str) -> anyhow::Result<Todo> {
        if message.is_empty() {
            return Err(anyhow!("no sense in an empty todo message"));
        }

        self.mutate(|list| list.add_message(message))
            .map(|(todo, _)| todo)
    }

    pub fn add(&self, todo: Todo) -> anyhow::Result<()> {
        self.mutate(|list| {
            list.add_todo(todo);
            Ok(())
        })
        .map(|(result, _)| result)
    }

    pub fn remove(&self, id: &str) -> anyhow::Result<()> {
        self.mutate(|list| list.remove(id))?;

        eprintln!("[INFO] removed a todo item");

        Ok(())
    }

    pub fn mark_done(&self, id: &str) -> anyhow::Result<()> {
        self.mutate(|list| list.mark_done(id))
            .map(|(result, _)| result)
    }

    pub fn remove_done(&self) -> anyhow::Result<()> {
        self.mutate(|list| {
            list.remove_done();
            Ok(())
        })
        .map(|(result, _)| result)
    }

    pub fn move_up(&self, id: String) -> anyhow::Result<()> {
        self.mutate(|list| list.move_up(id))
            .map(|(result, _)| result)
    }

    pub fn move_down(&self, id: String) -> anyhow::Result<()> {
        self.mutate(|list| list.move_down(id))
            .map(|(result, _)| result)
    }

    pub fn move_below(&self, id: &str, target_id: &str) -> anyhow::Result<()> {
        self.mutate(|list| list.move_below(id, target_id))?;

        eprintln!("[INFO] move a todo item below another");

        Ok(())
    }

    pub fn get_all(&self) -> anyhow::Result<Vec<Todo>> {
        let all = self.db.get_all_todos()?;
        eprintln!("[TRACE] getting all {} todos", all.len());
        Ok(all)
    }

    pub fn flush(&self) -> anyhow::Result<Vec<Todo>> {
        self.get_all()
    }
}

pub mod inmem {
    use super::*;
    use std::sync::Mutex;

    #[derive(Debug, Default)]
    pub struct NoopDB(Mutex<Vec<Todo>>);

    impl TodosDatabase for NoopDB {
        fn get_all_todos(&self) -> anyhow::Result<Vec<Todo>> {
            Ok(self.0.lock().map_err(|err| anyhow!("{err}"))?.clone())
        }

        fn update_todos<F>(&self, update: F) -> anyhow::Result<Vec<Todo>>
        where
            F: FnOnce(Vec<Todo>) -> anyhow::Result<Vec<Todo>>,
        {
            let mut todos = self.0.lock().map_err(|err| anyhow!("{err}"))?;
            *todos = update(todos.clone())?;
            Ok(todos.clone())
        }
    }

    impl Todos<NoopDB> {
        pub fn new_inmemory() -> Todos<NoopDB> {
            Todos::new(NoopDB::default())
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn move_below_from_top_to_bottom() {
        let todos = Todos::new_inmemory();

        todos.add_message("1").unwrap();
        todos.add_message("2").unwrap();
        let target = todos.add_message("3").unwrap().id.0;
        todos.add_message("4").unwrap();
        let id = todos.add_message("5").unwrap().id.0;
        // now, todos = [5, 4, 3, 2, 1]

        todos.move_below(&id, &target).unwrap();

        let messages = todos
            .get_all()
            .unwrap()
            .into_iter()
            .map(|t| t.message)
            .collect::<Vec<_>>();

        assert_eq!(
            messages,
            vec![
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "5".to_string(),
                "4".to_string(),
            ]
        )
    }

    #[test]
    fn move_below_from_bottom_to_top() {
        let todos = Todos::new_inmemory();

        todos.add_message("1").unwrap();
        let id = todos.add_message("2").unwrap().id.0;
        todos.add_message("3").unwrap();
        todos.add_message("4").unwrap();
        let target = todos.add_message("5").unwrap().id.0;
        // now, todos = [5, 4, 3, 2, 1]

        todos.move_below(&id, &target).unwrap();

        let messages = todos
            .get_all()
            .unwrap()
            .into_iter()
            .map(|t| t.message)
            .collect::<Vec<_>>();

        assert_eq!(
            messages,
            vec![
                "1".to_string(),
                "3".to_string(),
                "4".to_string(),
                "5".to_string(),
                "2".to_string(),
            ]
        )
    }

    #[test]
    fn move_below_to_bottom() {
        let todos = Todos::new_inmemory();

        let target = todos.add_message("1").unwrap().id.0;
        todos.add_message("2").unwrap();
        todos.add_message("3").unwrap();
        todos.add_message("4").unwrap();
        let id = todos.add_message("5").unwrap().id.0;
        // now, todos = [5, 4, 3, 2, 1]

        todos.move_below(&id, &target).unwrap();

        let messages = todos
            .get_all()
            .unwrap()
            .into_iter()
            .map(|t| t.message)
            .collect::<Vec<_>>();

        assert_eq!(
            messages,
            vec![
                "1".to_string(),
                "5".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
            ]
        )
    }

    #[test]
    fn moving_first_up_and_last_down_are_noops() {
        let todos = Todos::new_inmemory();
        let first = todos.add_message("first").unwrap().id.0;
        let last = todos.add_message("last").unwrap().id.0;

        todos.move_up(first.into()).unwrap();
        todos.move_down(last.into()).unwrap();

        assert_eq!(
            todos
                .get_all()
                .unwrap()
                .into_iter()
                .map(|todo| todo.message)
                .collect::<Vec<_>>(),
            ["first", "last"]
        );
    }

    #[test]
    fn finds_installed_mynd_web_app() {
        let directory = tempfile::tempdir().unwrap();
        let desktop_file = directory.path().join("chrome-mynd-Default.desktop");
        std::fs::write(
            &desktop_file,
            "[Desktop Entry]\nName=Mynd\nExec=chromium --app-id=mynd\n",
        )
        .unwrap();

        assert_eq!(find_installed_web_app(directory.path()), Some(desktop_file));
    }

    #[test]
    fn ignores_unrelated_desktop_apps() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(
            directory.path().join("other.desktop"),
            "[Desktop Entry]\nName=Other\nExec=chromium --app-id=other\n",
        )
        .unwrap();

        assert_eq!(find_installed_web_app(directory.path()), None);
    }

    #[test]
    fn detects_started_and_failed_launch_commands() {
        assert!(command_started(Command::new("sleep").arg("1").spawn()));
        assert!(!command_started(
            Command::new("sh").arg("-c").arg("exit 1").spawn()
        ));
    }
}
