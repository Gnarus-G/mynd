use std::{
    fmt::Display,
    sync::{Mutex, MutexGuard},
    usize,
};

use anyhow::anyhow;
use chrono::{Local, TimeZone};
use collection::array::TodoArrayList;
use collection::TodoCollection;
use persist::{ActualTodosDB, TodosDatabase};
use serde::{Deserialize, Serialize};

mod collection;
mod config;
mod lang;
pub mod persist;

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
    list: Mutex<collection::array::TodoArrayList>,
    pub db: DB,
}

impl<DB: TodosDatabase> Todos<DB> {
    pub fn new(db: DB) -> Self {
        Self {
            list: Mutex::new(collection::array::TodoArrayList::new()),
            db,
        }
    }
}

impl Todos<ActualTodosDB> {
    pub fn load_up_with_persistor() -> Todos<ActualTodosDB> {
        let db = ActualTodosDB::default();
        let list = Mutex::new(TodoArrayList::from(db.get_all_todos().unwrap_or_default()));
        Todos { list, db }
    }
}

impl<DB: TodosDatabase> Todos<DB> {
    pub fn reload(&self) -> anyhow::Result<()> {
        eprintln!("[TRACE] reloading todos");
        let todos = self.db.get_all_todos()?;
        *(self.inner_list()?) = TodoArrayList::from(todos);
        Ok(())
    }

    fn inner_list(&self) -> anyhow::Result<MutexGuard<TodoArrayList>> {
        self.list
            .try_lock()
            .map_err(|err| anyhow!("{err}").context("failed to acquire lock on todos list"))
    }

    pub fn add_message(&self, message: &str) -> anyhow::Result<Todo> {
        if message.is_empty() {
            return Err(anyhow!("no sense in an empty todo message"));
        }

        let todo = self.inner_list()?.add_message(message)?;

        Ok(todo)
    }

    pub fn add(&self, todo: Todo) -> anyhow::Result<()> {
        self.inner_list()?.add_todo(todo);
        Ok(())
    }

    pub fn remove(&self, id: &str) -> anyhow::Result<()> {
        self.inner_list()?.remove(id)?;

        eprintln!("[INFO] removed a todo item");

        Ok(())
    }

    pub fn mark_done(&self, id: &str) -> anyhow::Result<()> {
        self.inner_list()?.mark_done(id)?;

        Ok(())
    }

    pub fn remove_done(&self) -> anyhow::Result<()> {
        self.inner_list()?.remove_done();
        self.flush()?;

        Ok(())
    }

    pub fn move_up(&self, id: String) -> anyhow::Result<()> {
        self.inner_list()?.move_up(id)?;

        self.flush()?;

        Ok(())
    }

    pub fn move_down(&self, id: String) -> anyhow::Result<()> {
        self.inner_list()?.move_down(id)?;

        self.flush()?;

        Ok(())
    }

    pub fn move_below(&self, id: &str, target_id: &str) -> anyhow::Result<()> {
        self.inner_list()?.move_below(id, target_id)?;

        eprintln!("[INFO] move a todo item below another");

        self.flush()?;

        Ok(())
    }

    pub fn get_all(&self) -> anyhow::Result<Vec<Todo>> {
        let all = self.inner_list()?.get_all();
        eprintln!("[TRACE] getting all {} todos", all.len());
        Ok(all)
    }

    pub fn flush(&self) -> anyhow::Result<Vec<Todo>> {
        let all = self.get_all()?;
        self.db.set_all_todos(all.clone())?;
        Ok(all)
    }
}

pub mod inmem {
    use super::*;

    pub struct NoopDB;

    impl TodosDatabase for NoopDB {
        fn get_all_todos(&self) -> anyhow::Result<Vec<Todo>> {
            return Ok(vec![]);
        }

        fn set_all_todos(&self, _todos: Vec<Todo>) -> anyhow::Result<()> {
            Ok(())
        }
    }

    impl Todos<NoopDB> {
        pub fn new_inmemory() -> Todos<NoopDB> {
            Todos::new(NoopDB)
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
}
