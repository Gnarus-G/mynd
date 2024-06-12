use crate::Todo;

pub trait TodoCollection {
    fn add(&mut self, message: &str) -> anyhow::Result<Todo>;

    fn remove(&mut self, id: String) -> anyhow::Result<()>;

    fn len(&self) -> usize;

    fn mark_done(&mut self, id: String) -> anyhow::Result<()>;

    fn remove_done(&mut self);

    fn move_up(&mut self, id: String) -> anyhow::Result<()>;

    fn move_down(&mut self, id: String) -> anyhow::Result<()>;

    /// Move a todo item to be directly below another.
    fn move_below(&mut self, id: String, target_id: String) -> anyhow::Result<()>;

    fn get_all(&self) -> Vec<Todo>;
}

pub mod array {
    use anyhow::{anyhow, Context};

    use crate::{Todo, TodoID};

    #[derive(Debug)]
    pub struct TodoArrayList {
        list: Vec<Todo>,
    }

    impl TodoArrayList {
        pub fn new() -> Self {
            Self { list: vec![] }
        }

        fn find_index(&self, id: String) -> anyhow::Result<usize> {
            let idx = self
                .list
                .iter()
                .enumerate()
                .find(|(_, t)| t.id == TodoID(id.clone()))
                .context("didn't find a todo by the id provided")?
                .0;

            Ok(idx)
        }
    }

    impl From<Vec<Todo>> for TodoArrayList {
        fn from(value: Vec<Todo>) -> Self {
            Self { list: value }
        }
    }

    impl super::TodoCollection for TodoArrayList {
        fn add(&mut self, message: &str) -> anyhow::Result<Todo> {
            let todo = Todo::new(message.to_string());

            if self.list.iter().any(|i| i.id == todo.id) {
                return Err(anyhow!(
                    "[INFO] already noted this todo message: '{}'",
                    message
                ));
            }

            self.list.push(todo.clone());

            Ok(todo)
        }

        fn remove(&mut self, id: String) -> anyhow::Result<()> {
            let index = self.find_index(id)?;

            self.list.remove(index);

            Ok(())
        }

        fn len(&self) -> usize {
            self.list.len()
        }

        fn mark_done(&mut self, id: String) -> anyhow::Result<()> {
            let idx = self.find_index(id)?;

            let todo = self.list.get_mut(idx);

            if let Some(todo) = todo {
                todo.done = !todo.done;
            }

            Ok(())
        }

        fn remove_done(&mut self) {
            let copy = self.get_all();
            self.list = copy.iter().filter(|t| !t.done).cloned().collect();
        }

        fn move_up(&mut self, id: String) -> anyhow::Result<()> {
            let idx = self.find_index(id)?;

            if idx < self.len() {
                let curr = self.list[idx].clone();
                let temp = self.list[idx - 1].clone();

                self.list[idx] = temp;
                self.list[idx - 1] = curr;
            }

            Ok(())
        }

        fn move_down(&mut self, id: String) -> anyhow::Result<()> {
            let idx = self.find_index(id)?;

            if idx < self.len() {
                let curr = self.list[idx].clone();
                let temp = self.list[idx + 1].clone();

                self.list[idx] = temp;
                self.list[idx + 1] = curr;
            }

            Ok(())
        }

        /// Move a todo item to be directly below another.
        fn move_below(&mut self, id: String, target_id: String) -> anyhow::Result<()> {
            // remember here that todos are added to the front of the list
            // so 0..len is from most newest to oldest, top to bottom
            // so i + 1 is below i

            let idx = self.find_index(id)?;
            let target_idx = self.find_index(target_id)?;
            let below_target_idx = target_idx + 1;

            // wouldn't make a difference if todo is own target or already below target
            if idx == target_idx {
                return Err(anyhow!("[INFO] noop: won't move a todo item below itself"));
            }

            if idx == below_target_idx {
                return Err(anyhow!("[INFO] noop: todo is already below target"));
            }

            let size = self.len();

            if idx >= size || target_idx >= size {
                return Err(anyhow!(
                    "[WARN] tried to move todo item below another but one of them doesn't exist"
                ));
            }

            let source = self.list[idx].clone();

            if idx < target_idx {
                self.list.remove(idx);
                self.list.insert(target_idx, source);
            } else {
                self.list.remove(idx);
                self.list.insert(below_target_idx, source);
            }

            Ok(())
        }

        fn get_all(&self) -> Vec<Todo> {
            self.list.clone()
        }
    }
}

pub mod linkedlist {}
