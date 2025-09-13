use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path;
use std::path::Path;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{errors, fen, pgn};
use static_init::dynamic;
use std::sync::Mutex;

#[dynamic]
static ARCHIVE_DIRS: Mutex<HashMap<String, ()>> = Mutex::new(HashMap::new());


impl<T: FromStr + ToString> GameArchive<T> {
    fn try_new(dir_name: &str, file_extension: &str) -> Result<Self, &'static str> {
        let mut dirs = ARCHIVE_DIRS.lock().unwrap();
        if dirs.contains_key(dir_name) {
            return Err("Archive with this dir_name already exists");
        }
        dirs.insert(dir_name.to_string(), ());
        Ok(Self {
            data: Vec::new(),
            dir_name: dir_name.to_string(),
            file_extension: file_extension.to_string(),
        })
    }
}

const CONFIG_FILE_NAME: &str = "chess_oxide.toml";

#[cfg(target_os = "windows")]
const CONFIG_DIR: &str = "C:\\Users\\%USERNAME%\\AppData\\Local\\chess_oxide";
#[cfg(target_os = "linux")]
const CONFIG_DIR: &str = "/home/$USER/.chess_oxide";
#[cfg(target_os = "macos")]
const CONFIG_DIR: &str = "/Users/$USER/Library/Application Support/chess_oxide";

#[cfg(target_os = "windows")]
const ARCHIVE_DIR: &str = "C:\\Users\\%USERNAME%\\AppData\\Local\\chess_oxide\\archives";
#[cfg(target_os = "linux")]
const ARCHIVE_DIR: &str = "/home/$USER/.chess_oxide/archives";
#[cfg(target_os = "macos")]
const ARCHIVE_DIR: &str = "/Users/$USER/Library/Application Support/chess_oxide/archives";

struct Config {}

// pub trait Archive<T> {
//     fn find_by_name(&self) -> T;
//     fn find_by_id(&self) -> T;
//     fn save(&self, name: &str);
//     fn delete(&self, id: i32);
// }

pub struct ArchiveEntry<T> {
    data: T,
    name: String,
}

pub struct GameArchive<T> {
    data: Vec<ArchiveEntry<T>>,
    dir_name: String,
    file_extension: String,
}
impl<T: FromStr + ToString> GameArchive<T> {
    fn new(dir_name: &str, file_extension: &str) -> Self {
        Self {
            data: Vec::new(),
            dir_name: dir_name.to_string(),
            file_extension: file_extension.to_string(),
        }
    }

    fn find_by_name(&self, name: &str) -> Option<&T> {
        for entry in &self.data {
            if entry.name == name {
                return Some(&entry.data);
            }
        }
        None
    }

    fn find_by_idx(&self, idx: usize) -> Option<&T> {
        if let Some(entry) = self.data.get(idx) {
            Some(&entry.data)
        } else {
            None
        }
    }

    fn find_idx(&self, name: &str) -> Option<usize> {
        for (idx, entry) in self.data.iter().enumerate() {
            if entry.name == name {
                return Some(idx);
            }
        }
        None
    }

    fn save(&mut self, name: &str, data: T) {
        self.data.push(ArchiveEntry {
            data,
            name: name.to_string(),
        });
    }

    fn delete_by_idx(&self, idx: i32) {}

    fn get_path(&self) -> path::PathBuf {
        Path::new(ARCHIVE_DIR).join(&self.dir_name)
    }

    fn read<E>(&mut self) -> Result<(), E>
    where
        E: From<io::Error> + From<<T as FromStr>::Err>,
    {
        let files = fs::read_dir(self.get_path()).map_err(E::from)?;
        for f in files {
            let file = f.map_err(E::from)?;
            if file.file_type().map_err(E::from)?.is_file()
                && file.path().extension() == Some(self.file_extension.as_str().as_ref())
            {
                let mut content = String::new();
                fs::File::open(file.path())
                    .map_err(E::from)?
                    .read_to_string(&mut content)
                    .map_err(E::from)?;
                // parse the content into the specified type T
                let data = T::from_str(&content).map_err(E::from)?;
                self.data.push(ArchiveEntry {
                    data,
                    name: file.file_name().to_string_lossy().into_owned(),
                });
            }
        }
        Ok(())
    }

    fn write(&self) -> Result<(), io::Error> {
        fs::create_dir_all(self.get_path())?;
        for entry in &self.data {
            let file_path = self
                .get_path()
                .join(format!("{}.{}", entry.name, self.file_extension));
            let mut file = fs::File::create(file_path)?;
            file.write_all(entry.data.to_string().as_bytes())?;
        }
        Ok(())
    }
}
