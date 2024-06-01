use std::{error::Error, sync::Mutex};

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
    pub fn reload(&self) {
        *self.list.lock().unwrap() = self.db.get_all_todos();
    }

    pub fn add(&self, message: &str) -> Result<Todo, Box<dyn Error>> {
        let mut list = self.list.lock().unwrap();
        let todo = Todo::new(message.to_string());
        list.insert(0, todo);

        self.update();

        Ok(list[0].clone())
    }

    fn size(&self) -> usize {
        self.list.lock().expect("mutex is in a bad state").len()
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

        self.update();
    }

    pub fn remove_done(&self) {
        let copy = self.get_all();
        *self.list.lock().unwrap() = copy.iter().filter(|t| !t.done).cloned().collect();
        self.update();
    }

    pub fn move_up(&self, id: String) {
        let idx = self.find_index(id);

        if idx < self.list.lock().unwrap().len() {
            let curr = self.list.lock().unwrap()[idx].clone();
            let temp = self.list.lock().unwrap()[idx - 1].clone();

            self.list.lock().unwrap()[idx] = temp;
            self.list.lock().unwrap()[idx - 1] = curr;

            self.update();
        }
    }

    pub fn move_down(&self, id: String) {
        let idx = self.find_index(id);

        if idx < self.list.lock().unwrap().len() {
            let curr = self.list.lock().unwrap()[idx].clone();
            let temp = self.list.lock().unwrap()[idx + 1].clone();

            self.list.lock().unwrap()[idx] = temp;
            self.list.lock().unwrap()[idx + 1] = curr;

            self.update();
        }
    }

    /// Move a todo item to be directly below another.
    pub fn move_below(&self, id: String, target_id: String) {
        // remember here that todos are added to the front of the list
        // so 0..len is from most newest to oldest, top to bottom
        // so i + 1 is below i

        let idx = self.find_index(id);
        let target_idx = self.find_index(target_id);
        let below_target_idx = target_idx + 1;

        // wouldn't make a difference if todo is own target or already below target
        if idx == target_idx {
            eprintln!("[INFO] noop: won't move a todo item below itself");
            return;
        }

        if idx == below_target_idx {
            eprintln!("[INFO] noop: todo is already below target");
            return;
        }

        if idx >= self.size() || target_idx >= self.size() {
            eprintln!("[WARN] tried to move todo item below another but one of them doesn't exist");
            return; // TODO: error, bad input
        }

        let source = self.list.lock().unwrap()[idx].clone();

        if idx < target_idx {
            self.list.lock().unwrap().remove(idx);
            self.list.lock().unwrap().insert(target_idx, source);
        } else {
            self.list.lock().unwrap().remove(idx);
            self.list.lock().unwrap().insert(below_target_idx, source);
        }

        eprintln!("[INFO] move a todo item below another");

        self.update();
    }

    pub fn get_all(&self) -> Vec<Todo> {
        self.list.lock().unwrap().clone()
    }

    fn update(&self) {
        self.db.set_all_todos(self.get_all())
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

        todos.move_below(id, target);

        let messages = todos
            .get_all()
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

        todos.move_below(id, target);

        let messages = todos
            .get_all()
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

        todos.move_below(id, target);

        let messages = todos
            .get_all()
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
