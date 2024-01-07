use serde::{Deserialize, Serialize};

use std::path::Path;
use std::{collections::HashMap, error::Error, fs};


use crate::task::wrapper::GenericTaskWrapper;
use crate::task::{match_task::MatchTask};
use crate::task::builtins::{BuiltinTask, test_tasks};

#[cfg(test)]
mod test {
    use std::{error::Error, fs::OpenOptions, io::Write};

    use super::*;

    #[test]
    fn test_load_config() {
        let config = TaskConfig::load("../../resources").unwrap();
        println!("{:#?}", config)
    }

    #[test]
    fn write_default_task_config() -> Result<(), Box<dyn Error>> {
        let mut open_options = OpenOptions::new();
        open_options.write(true).create(true);
        let config = TaskConfig::default();
        let config_file = "../../resources/tasks.toml";

        {
            println!("{:?}", config);
            let config = toml::to_string_pretty(&config)?;
            println!("{}", config);
            let mut file = open_options.open(config_file)?;
            file.write_fmt(format_args!("{}", config))?;
        }

        // {
        //     let config = serde_json::to_string_pretty(&config)?;
        //     let config_file = "./resources/tasks.json";
        //     let mut file = open_options.open(config_file)?;
        //     file.write_fmt(format_args!("{}", config))?;
        // }

        // {
        //     let config = fs::read_to_string(config_file)?;
        //     let config: TaskConfig = toml::from_str(&config)?;
        //     println!("{:?}", config);
        // }
        Ok(())
    }

    #[test]
    fn test_load_task_config() -> Result<(), Box<dyn Error>> {
        let task = TaskConfig::load("../../resources")?;
        println!("{:?}", task);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskConfig(pub HashMap<String, BuiltinTask>);
impl TaskConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let path = path.as_ref();
        let task_config = path.join("tasks.toml");
        println!("{:?}", task_config);
        let task_config = fs::read_to_string(task_config)?;
        let mut task_config = toml::from_str::<TaskConfig>(&task_config)?;

        if let Ok(read_dir) = fs::read_dir(path.join("tasks")) {
            for entry in read_dir {
                let entry = entry.unwrap();
                if entry.path().extension().and_then(|s| s.to_str()) != Some("toml") {
                    continue;
                }
                if let Ok(task) = fs::read_to_string(entry.path()) {
                    let task = toml::from_str::<BuiltinTask>(&task)?;

                    task_config.0.insert(
                        entry
                            .path()
                            .file_prefix()
                            .and_then(|s| s.to_str())
                            .unwrap()
                            .to_string(),
                        task,
                    );
                }
            }
        }
        Ok(task_config)
    }

    pub fn get_task<S: AsRef<str>>(&self, name: S) -> Result<BuiltinTask, String> {
        return self
            .0
            .get(name.as_ref())
            .ok_or("failed to retrive task from task_config".to_string())
            .map(|task| task.clone());
    }
}

impl Default for TaskConfig {
    fn default() -> Self {
        let mut map = HashMap::new();
        let test_tasks = test_tasks();
        for (name, task) in test_tasks {
            map.insert(name.to_string(), task);
        }
        Self(map)
    }
}

