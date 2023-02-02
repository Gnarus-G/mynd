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
        let filename = [self.file_prefix, item.id()].join("-");
        let path = self.dir.join(&filename).with_extension("json");
        let json = serde_json::to_string::<Item>(&item)?;
        let mut file = self.open_file(&path)?;
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
}
