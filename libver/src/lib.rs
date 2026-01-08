//! > *Basic runtime version management library*
//!
//! This is the documentation for `libver`, which is the main backbone for
//! [`verune`](https://codeberg.org/r6915ee/verune).
//!
//! Please note that this documentation is fairly empty and does not contain a
//! lot of information. To see how each method is used, please see the `verune`
//! source code.

use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    env::{self, VarError, home_dir},
    fmt::Display,
    fs::read_to_string,
    io::{Error, ErrorKind, Result as IoResult},
    path::PathBuf,
    process::Command,
};

/// Basic metadata representation for a [Runtime].
///
/// [Runtime]s will typically be composed of this struct in order to allow
/// dependent crates to access additional data about a [Runtime], as well as
/// access the search paths.
#[derive(PartialEq, Default, Eq, Hash, Deserialize, Serialize)]
pub struct RuntimeMetadata {
    pub display_name: String,
    pub search_paths: Vec<String>,
}

/// Basic I/O layer for a runtime.
///
/// This structure both contains data about a runtime and keeps track of its
/// metadata. It provides various I/O operations on runtimes in a consistent
/// manner.
#[derive(PartialEq, Eq, Hash)]
pub struct Runtime {
    pub name: String,
    pub metadata: RuntimeMetadata,
}

impl Runtime {
    /// Creates a [Runtime] with default metadata options.
    pub fn unsafe_new<T: Display>(name: T) -> Runtime {
        Runtime {
            name: name.to_string(),
            metadata: RuntimeMetadata::default(),
        }
    }

    /// Attempts to create a [Runtime].
    ///
    /// Note that this method can fail in the following cases:
    ///
    /// - The home directory is inaccessible
    /// - The metadata file can't be read
    /// - The metadata file can't be deserialized
    pub fn new<T: Display>(name: T) -> IoResult<Runtime> {
        let mut buf: PathBuf = Runtime::get_runtime(&name)?;
        buf.push("meta.ron");
        let data: String = read_to_string(buf)?;
        match ron::from_str::<RuntimeMetadata>(data.as_str()) {
            Ok(metadata) => Ok(Runtime {
                name: name.to_string(),
                metadata,
            }),
            Err(_) => Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Metadata file for runtime \"{}\" is not valid runtime metadata",
                    name
                ),
            )),
        }
    }

    /// Gets the root directory for a [Runtime], and returns it if successful.
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

    /// Gets a [Runtime] directory, based on its name.
    pub fn get_runtime<T: Display>(name: T) -> IoResult<PathBuf> {
        let mut buf: PathBuf = Runtime::get_root()?;
        buf.push(name.to_string());
        Ok(buf)
    }

    /// Gets a version directory from the current [Runtime].
    pub fn get_version<T: Display>(&self, version: T) -> IoResult<PathBuf> {
        let mut buf: PathBuf = Runtime::get_runtime(&self.name)?;
        buf.push(version.to_string());
        Ok(buf)
    }

    /// Checks if a version directory exists, returning it if it does.
    pub fn get_safe_version<T: Display>(&self, version: T) -> IoResult<PathBuf> {
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

    /// Gets the list of version search paths as relative paths to the version
    /// directory, checking if each one exists.
    pub fn get_version_search_paths<T: Display>(&self, version: T) -> IoResult<Vec<PathBuf>> {
        let mut paths: Vec<PathBuf> = Vec::new();
        let version_dir: PathBuf = self.get_safe_version(version)?;
        for search_path in &self.metadata.search_paths {
            let search_buf: PathBuf = search_path.into();
            let proper_search_buf: PathBuf = version_dir.join(search_buf);
            if proper_search_buf.try_exists()? {
                paths.push(proper_search_buf);
            } else {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!(
                        "Search path \"{}\" does not exist",
                        proper_search_buf.display()
                    ),
                ));
            }
        }
        Ok(paths)
    }
}

pub mod conf {
    use crate::*;
    use std::{
        io::{Error, ErrorKind, Result as IoResult},
        path::Path,
    };

    /// Reads a configuration file, and then parses it.
    pub fn parse<T: AsRef<Path>>(path: T) -> IoResult<HashMap<String, String>> {
        let data: String = read_to_string(path.as_ref())?;
        match ron::from_str::<HashMap<String, String>>(data.as_str()) {
            Ok(map) => Ok(map),
            Err(_) => Err(Error::new(
                ErrorKind::InvalidData,
                "Configuration file is invalid",
            )),
        }
    }

    /// Returns a transformed variant of a configuration file using
    /// [Runtime::unsafe_new].
    pub fn unsafe_collect(data: HashMap<String, String>) -> HashMap<Runtime, String> {
        let mut parsed: HashMap<Runtime, String> = HashMap::new();
        for (name, value) in data.iter() {
            let runtime: Runtime = Runtime::unsafe_new(name.to_string());
            parsed.insert(runtime, value.to_string());
        }
        parsed
    }

    /// Returns a transformed variant of a configuration file using [Runtime::new].
    pub fn collect(data: HashMap<String, String>) -> IoResult<HashMap<Runtime, String>> {
        let mut parsed: HashMap<Runtime, String> = HashMap::new();
        for (name, value) in data.iter() {
            let runtime: Runtime = Runtime::new(name.to_string())?;
            parsed.insert(runtime, value.to_string());
        }
        Ok(parsed)
    }
}

/// Executes a program with an environment suited to various runtime versions.
///
/// This method is the main backbone for commands to run under version-managed
/// scenarios. `args` can be used as both a way to specify the command and as
/// a way to specify the arguments, though it will fallback to system defaults
/// if `args` is empty.
pub fn exec<T: Into<VecDeque<String>>>(
    args: T,
    config: HashMap<Runtime, String>,
) -> IoResult<Command> {
    let mut deque: VecDeque<String> = args.into();
    let mut cmd: Command = Command::new(if let Some(data) = deque.pop_front() {
        data
    } else if let Ok(shell) = env::var("SHELL") {
        shell
    } else if cfg!(windows) {
        "cmd".into()
    } else {
        "sh".into()
    });
    let mut paths: Vec<PathBuf> = Vec::with_capacity(config.len());
    for (runtime, version) in config.iter() {
        paths.push(runtime.get_safe_version(version)?);
        paths.extend(runtime.get_version_search_paths(version)?);
    }
    let path: Result<String, VarError> = env::var("PATH");
    cmd.args(deque)
        .env("PATH", {
            let mut iter = paths.iter();
            let mut prefix: String = String::new();
            if let Some(path) = iter.next()
                && let Some(data) = path.to_str()
            {
                prefix.push_str(data);
            }

            let delim: char = if cfg!(windows) { ';' } else { ':' };
            for path in iter {
                if let Some(data) = path.to_str() {
                    prefix.push(delim);
                    prefix.push_str(data);
                }
            }
            if let Ok(good_path) = path {
                prefix.push(delim);
                prefix.push_str(good_path.as_str());
            }
            prefix
        })
        .env("VER_SCOPE", {
            if let Ok(last_scope) = env::var("VER_SCOPE")
                && let Ok(data) = last_scope.parse::<usize>()
            {
                data + 1
            } else {
                1
            }
            .to_string()
        })
        // $VER_OVERRIDE only remains for legacy purposes. It'll be removed in a later commit.
        .env("VER_OVERRIDE", "1");
    Ok(cmd)
}
