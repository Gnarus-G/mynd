use std::sync::{Mutex, MutexGuard};

use anyhow::{anyhow, Context};
use persist::{jsonfile::TodosJsonDB, TodosDatabase};
use serde::{Deserialize, Serialize};

pub mod persist;

#[derive(Debug, Deserialize, Serialize, PartialEq, PartialOrd, Clone)]
pub struct TodoID(pub String);
impl TodoID {
    fn hash_message(message: &str) -> TodoID {
        TodoID(sha256::digest(message))
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
            id: TodoID::hash_message(&message),
            message,
            created_at: Default::default(),
            done: false,
        }
    }
}

#[derive(Debug)]
pub struct Todos<DB: TodosDatabase> {
    list: Mutex<Vec<Todo>>,
    db: DB,
}

impl Todos<TodosJsonDB> {
    pub fn load_up_with_persistor() -> Todos<TodosJsonDB> {
        let db = TodosJsonDB::default();
        Todos {
            list: Mutex::new(db.get_all_todos()),
            db,
        }
    }
}

impl<DB: TodosDatabase> Todos<DB> {
    pub fn reload(&self) -> anyhow::Result<()> {
        *(self.inner_list()?) = self.db.get_all_todos();
        Ok(())
    }

    fn inner_list(&self) -> anyhow::Result<MutexGuard<Vec<Todo>>> {
        self.list
            .try_lock()
            .map_err(|err| anyhow!("{err}").context("failed to acquire lock on todos list"))
    }

    pub fn add(&self, message: &str) -> anyhow::Result<Todo> {
        let todo = Todo::new(message.to_string());

        self.inner_list()?.insert(0, todo.clone());

        self.update()?;

        Ok(todo)
    }

    fn size(&self) -> usize {
        self.inner_list()
            .expect("failed to get inner todo list to check size")
            .len()
    }

    fn find_index(&self, id: String) -> anyhow::Result<usize> {
        let idx = self
            .inner_list()?
            .iter()
            .enumerate()
            .find(|(_, t)| t.id == TodoID(id.clone()))
            .context("didn't find a todo by the id provided")?
            .0;

        Ok(idx)
    }

    pub fn mark_done(&self, id: TodoID) -> anyhow::Result<()> {
        let idx = self.find_index(id.0)?;
        let mut list = self.inner_list()?;
        let todo = list.get_mut(idx);

        if let Some(todo) = todo {
            todo.done = !todo.done;
        }

        self.update()?;

        Ok(())
    }

    pub fn remove_done(&self) -> anyhow::Result<()> {
        let copy = self.get_all()?;
        *(self.inner_list()?) = copy.iter().filter(|t| !t.done).cloned().collect();
        self.update()?;

        Ok(())
    }

    pub fn move_up(&self, id: String) -> anyhow::Result<()> {
        let idx = self.find_index(id)?;

        if idx < self.inner_list()?.len() {
            let curr = self.inner_list()?[idx].clone();
            let temp = self.inner_list()?[idx - 1].clone();

            self.inner_list()?[idx] = temp;
            self.inner_list()?[idx - 1] = curr;

            self.update()?;
        }

        Ok(())
    }

    pub fn move_down(&self, id: String) -> anyhow::Result<()> {
        let idx = self.find_index(id)?;

        if idx < self.inner_list()?.len() {
            let curr = self.inner_list()?[idx].clone();
            let temp = self.inner_list()?[idx + 1].clone();

            self.inner_list()?[idx] = temp;
            self.inner_list()?[idx + 1] = curr;

            self.update()?;
        }

        Ok(())
    }

    /// Move a todo item to be directly below another.
    pub fn move_below(&self, id: String, target_id: String) -> anyhow::Result<()> {
        // remember here that todos are added to the front of the list
        // so 0..len is from most newest to oldest, top to bottom
        // so i + 1 is below i

        let idx = self.find_index(id)?;
        let target_idx = self.find_index(target_id)?;
        let below_target_idx = target_idx + 1;

        // wouldn't make a difference if todo is own target or already below target
        if idx == target_idx {
            eprintln!("[INFO] noop: won't move a todo item below itself");
            return Ok(());
        }

        if idx == below_target_idx {
            eprintln!("[INFO] noop: todo is already below target");
            return Ok(());
        }

        if idx >= self.size() || target_idx >= self.size() {
            eprintln!("[WARN] tried to move todo item below another but one of them doesn't exist");
            return Ok(()); // TODO: error, bad input
        }

        let source = self.inner_list()?[idx].clone();

        if idx < target_idx {
            self.inner_list()?.remove(idx);
            self.inner_list()?.insert(target_idx, source);
        } else {
            self.inner_list()?.remove(idx);
            self.inner_list()?.insert(below_target_idx, source);
        }

        eprintln!("[INFO] move a todo item below another");

        self.update()?;

        Ok(())
    }

    pub fn get_all(&self) -> anyhow::Result<Vec<Todo>> {
        let all = self.inner_list()?.clone();
        Ok(all)
    }

    fn update(&self) -> anyhow::Result<()> {
        let all = self.get_all()?;
        self.db.set_all_todos(all);
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    pub struct NoopDB;

    impl TodosDatabase for NoopDB {
        fn get_all_todos(&self) -> Vec<Todo> {
            return vec![];
        }

        fn set_all_todos(&self, _todos: Vec<Todo>) {}
    }

    impl Todos<NoopDB> {
        pub fn new() -> Todos<NoopDB> {
            Todos {
                list: Mutex::new(vec![]),
                db: NoopDB,
            }
        }
    }

    #[test]
    fn move_below_from_top_to_bottom() {
        let todos = Todos::new();

        todos.add("1").unwrap();
        todos.add("2").unwrap();
        let target = todos.add("3").unwrap().id.0;
        todos.add("4").unwrap();
        let id = todos.add("5").unwrap().id.0;
        // now, todos = [5, 4, 3, 2, 1]

        todos.move_below(id, target).unwrap();

        let messages = todos
            .get_all()
            .unwrap()
            .into_iter()
            .map(|t| t.message)
            .collect::<Vec<_>>();

        assert_eq!(
            messages,
            vec![
                "4".to_string(),
                "3".to_string(),
                "5".to_string(),
                "2".to_string(),
                "1".to_string(),
            ]
        )
    }

    #[test]
    fn move_below_from_bottom_to_top() {
        let todos = Todos::new();

        todos.add("1").unwrap();
        let id = todos.add("2").unwrap().id.0;
        todos.add("3").unwrap();
        todos.add("4").unwrap();
        let target = todos.add("5").unwrap().id.0;
        // now, todos = [5, 4, 3, 2, 1]

        todos.move_below(id, target).unwrap();

        let messages = todos
            .get_all()
            .unwrap()
            .into_iter()
            .map(|t| t.message)
            .collect::<Vec<_>>();

        assert_eq!(
            messages,
            vec![
                "5".to_string(),
                "2".to_string(),
                "4".to_string(),
                "3".to_string(),
                "1".to_string(),
            ]
        )
    }

    #[test]
    fn move_below_to_bottom() {
        let todos = Todos::new();

        let target = todos.add("1").unwrap().id.0;
        todos.add("2").unwrap();
        todos.add("3").unwrap();
        todos.add("4").unwrap();
        let id = todos.add("5").unwrap().id.0;
        // now, todos = [5, 4, 3, 2, 1]

        todos.move_below(id, target).unwrap();

        let messages = todos
            .get_all()
            .unwrap()
            .into_iter()
            .map(|t| t.message)
            .collect::<Vec<_>>();

        assert_eq!(
            messages,
            vec![
                "4".to_string(),
                "3".to_string(),
                "2".to_string(),
                "1".to_string(),
                "5".to_string(),
            ]
        )
    }
}
