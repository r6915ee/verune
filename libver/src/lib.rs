use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env::home_dir,
    fs::read_to_string,
    io::{Error, ErrorKind, Result as IoResult},
    path::PathBuf,
};

#[derive(PartialEq, Default, Eq, Hash, Deserialize, Serialize)]
pub struct RuntimeMetadata {
    pub display_name: String,
    pub search_paths: Vec<String>,
}

#[derive(PartialEq, Eq, Hash)]
pub struct Runtime {
    pub name: String,
    pub metadata: RuntimeMetadata,
}

impl Runtime {
    pub fn unsafe_new(name: String) -> Runtime {
        Runtime {
            name,
            metadata: RuntimeMetadata::default(),
        }
    }

    pub fn new(name: String) -> IoResult<Runtime> {
        let mut buf: PathBuf = Runtime::get_runtime(&name)?;
        buf.push("meta.ron");
        let data: String = read_to_string(buf)?;
        match ron::from_str::<RuntimeMetadata>(data.as_str()) {
            Ok(metadata) => Ok(Runtime { name, metadata }),
            Err(_) => Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Metadata file for runtime \"{}\" is not valid runtime metadata",
                    name
                ),
            )),
        }
    }

    pub fn get_root() -> IoResult<PathBuf> {
        if let Some(mut home) = home_dir() {
            home.push(".ver");
            Ok(home)
        } else {
            Err(Error::new(
                ErrorKind::NotFound,
                "Could not access home directory",
            ))
        }
    }

    pub fn get_runtime(name: &str) -> IoResult<PathBuf> {
        let mut buf: PathBuf = Runtime::get_root()?;
        buf.push(name);
        Ok(buf)
    }

    pub fn get_version(&self, version: String) -> IoResult<PathBuf> {
        let mut buf: PathBuf = Runtime::get_runtime(&self.name)?;
        buf.push(version);
        Ok(buf)
    }

    pub fn get_safe_version(&self, version: String) -> IoResult<PathBuf> {
        let path: PathBuf = self.get_version(version.to_string())?;
        if path.try_exists()? {
            Ok(path)
        } else {
            Err(Error::new(
                ErrorKind::NotFound,
                format!(
                    "Version {} for runtime \"{}\" was not found",
                    version, self.name
                ),
            ))
        }
    }
}

pub fn parse_config<T: AsRef<str>>(path: T) -> IoResult<HashMap<String, String>> {
    let data: String = read_to_string(path.as_ref())?;
    match ron::from_str::<HashMap<String, String>>(data.as_str()) {
        Ok(map) => Ok(map),
        Err(_) => Err(Error::new(
            ErrorKind::InvalidData,
            "Configuration file is invalid",
        )),
    }
}

pub fn unsafe_collect(data: HashMap<String, String>) -> HashMap<Runtime, String> {
    let mut parsed: HashMap<Runtime, String> = HashMap::new();
    for (name, value) in data.iter() {
        let runtime: Runtime = Runtime::unsafe_new(name.to_string());
        parsed.insert(runtime, value.to_string());
    }
    parsed
}

pub fn collect_config(data: HashMap<String, String>) -> IoResult<HashMap<Runtime, String>> {
    let mut parsed: HashMap<Runtime, String> = HashMap::new();
    for (name, value) in data.iter() {
        let runtime: Runtime = Runtime::new(name.to_string())?;
        parsed.insert(runtime, value.to_string());
    }
    Ok(parsed)
}
