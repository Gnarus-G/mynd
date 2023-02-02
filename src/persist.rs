use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io::{BufReader, Write},
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Serialize};

pub trait HasId {
    fn id(&self) -> &str;
}

#[derive(Debug)]
pub struct PersistenJson<'a> {
    file_prefix: &'a str,
    dir: PathBuf,
}

impl<'a> PersistenJson<'a> {
    pub fn new(dirname: &str, file_prefix: &'a str) -> Result<Self, Box<dyn Error>> {
        let dir = Path::new(&std::env::var("HOME")?).join(dirname);
        if !dir.is_dir() {
            fs::create_dir(&dir)?;
        }

        Ok(Self { dir, file_prefix })
    }

    pub fn add<Item: DeserializeOwned + Serialize + HasId>(
        &mut self,
        item: Item,
    ) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string::<Item>(&item)?;
        let mut file = self.open_file(&self.get_filename(item.id()))?;
        write!(file, "{}", json)?;
        Ok(())
    }

    fn open_file(&self, path: &Path) -> Result<File, Box<dyn Error>> {
        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(path)?;

        Ok(file)
    }

    fn get_filename(&self, id: &str) -> PathBuf {
        let basename = [self.file_prefix, id].join("-");
        self.dir.join(basename).with_extension("json")
    }

    fn read_item<Item: DeserializeOwned + Serialize>(
        &self,
        file: File,
    ) -> Result<Item, Box<dyn Error>> {
        let reader = BufReader::new(&file);
        let item = serde_json::from_reader(reader)?;
        Ok(item)
    }

    pub fn items<Item: DeserializeOwned + Serialize>(&self) -> Result<Vec<Item>, Box<dyn Error>> {
        let items = fs::read_dir(&self.dir)?
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| match path.extension() {
                Some(ex) if ex == "json" => true,
                _ => false,
            })
            .map(|path| self.open_file(&path))
            .flatten()
            .map(|file| self.read_item(file))
            .flatten()
            .collect();

        Ok(items)
    }

    pub fn remove_all(&self, ids: &[String]) {
        for path in ids.iter().map(|id| self.get_filename(id)) {
            match fs::remove_file(&path) {
                Err(e) => eprintln!("couldn't remove {path:?}; {e}"),
                _ => {}
            }
        }
    }
}
