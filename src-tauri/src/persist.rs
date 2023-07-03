use std::{
    error::Error,
    fs::{File, OpenOptions},
    io::{BufReader, Write},
    path::Path,
};

use serde::{de::DeserializeOwned, Serialize};

pub fn read_json<Item: DeserializeOwned + Serialize>(
    filename: &str,
) -> Result<Item, Box<dyn Error>> {
    let p = &std::env::var("HOME")?;
    let file = open_file(&Path::new(p).join(filename))?;
    let reader = BufReader::new(&file);
    let item = serde_json::from_reader(reader)?;
    Ok(item)
}

pub fn write_json<Item: DeserializeOwned + Serialize>(
    filename: &str,
    item: Item,
) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string::<Item>(&item)?;
    let p = &std::env::var("HOME")?;

    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .truncate(true)
        .open(Path::new(p).join(filename))?;

    write!(file, "{}", json)?;
    Ok(())
}

fn open_file(path: &Path) -> Result<File, Box<dyn Error>> {
    let file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(path)?;

    Ok(file)
}
