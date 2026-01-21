use std::{collections::HashMap, fs, ops::Deref, path::Path};

use serde::{Deserialize, Serialize};

use crate::actions::{Action, Task};

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskConfig(pub HashMap<String, Task<Action>>);

impl Deref for TaskConfig {
    type Target = HashMap<String, Task<Action>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TaskConfig {
    /// Recursively load all TOML task files from a directory.
    ///
    /// Naming convention:
    /// - tasks/foo.toml -> key = "foo"
    /// - tasks/dir/bar.toml -> key = "dir/bar" (if name field is empty)
    ///
    /// Priority: task.name > relative file path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
        let path = path.as_ref();
        let mut config = TaskConfig(HashMap::new());

        Self::load_recursive(path, path, &mut config.0)?;

        Ok(config)
    }

    /// Recursive helper function to load tasks from directories.
    ///
    /// - root: root directory (tasks/)
    /// - current: current directory being scanned
    /// - map: HashMap to store tasks
    fn load_recursive(
        root: &Path,
        current: &Path,
        map: &mut HashMap<String, Task<Action>>,
    ) -> Result<(), anyhow::Error> {
        if let Ok(read_dir) = fs::read_dir(current) {
            for entry in read_dir {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    // Recursively process subdirectories
                    Self::load_recursive(root, &path, map)?;
                } else if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    // Load TOML file
                    match fs::read_to_string(&path) {
                        Ok(content) => match toml::from_str::<Task<Action>>(&content) {
                            Ok(task) => {
                                // Use task.name as key (priority)
                                // If empty, use relative file path
                                let key = if !task.name.is_empty() {
                                    task.name.clone()
                                } else {
                                    path.strip_prefix(root)
                                        .unwrap_or(&path)
                                        .with_extension("")
                                        .to_string_lossy()
                                        .replace('\\', "/")
                                };

                                // Check for duplicate key and warn
                                if map.contains_key(&key) {
                                    eprintln!(
                                        "Warning: Duplicate task name '{}' found in {:?}, previous task will be overwritten",
                                        key, path
                                    );
                                }

                                map.insert(key, task);
                            }
                            Err(e) => {
                                eprintln!(
                                    "Failed to parse task file {:?}: {}",
                                    path, e
                                );
                            }
                        },
                        Err(e) => {
                            eprintln!(
                                "Failed to read task file {:?}: {}",
                                path, e
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_tasks() {
        // This test requires actual resource directory
        // In practice, use the resource loading to test
    }
}
