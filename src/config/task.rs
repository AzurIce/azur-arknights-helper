use serde::{Deserialize, Serialize};

use std::{collections::HashMap, fs::{self, File}, path::PathBuf, error::Error};

use crate::task::{MatchTask, self, Exec};

#[cfg(test)]
mod test {
    use std::{
        error::Error,
        fs::{self, File, OpenOptions},
        io::Write,
    };

    use super::*;

    #[test]
    fn test_task_config() -> Result<(), Box<dyn Error>> {
        let mut open_options = OpenOptions::new();
        open_options.write(true).create(true);
        let config = TaskConfig::default();
        let config_file = "./resources/tasks.toml";

        {
            let config = toml::to_string_pretty(&config)?;
            let mut file = open_options.open(config_file)?;
            file.write_fmt(format_args!("{}", config))?;
        }

        // {
        //     let config = serde_json::to_string_pretty(&config)?;
        //     let config_file = "./resources/tasks.json";
        //     let mut file = open_options.open(config_file)?;
        //     file.write_fmt(format_args!("{}", config))?;
        // }

        {
            let config = fs::read_to_string(config_file)?;
            let config: TaskConfig = toml::from_str(&config)?;
            println!("{:?}", config);
        }
        Ok(())
    }

    #[test]
    fn test_load_task_config() -> Result<(), Box<dyn Error>> {
        let task = TaskConfig::load()?;
        println!("{:?}", task);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskConfig(pub HashMap<String, BuiltinTask>);
impl TaskConfig {
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let task_config = fs::read_to_string("./resources/tasks.toml")?;
        let mut task_config = toml::from_str::<TaskConfig>(&task_config)?;

        if let Ok(read_dir) = fs::read_dir(PathBuf::from("./resources/tasks/")) {
            for entry in read_dir {
                let entry = entry.unwrap();
                if entry.path().extension().and_then(|s| s.to_str()) != Some("toml") {
                    continue;
                }
                if let Ok(task) = fs::read_to_string(entry.path()) {
                    if let Ok(task) = toml::from_str::<BuiltinTask>(&task) {
                        task_config.0.insert(
                            entry.path().file_prefix().and_then(|s| s.to_str()).unwrap().to_string(),
                            task,
                        );
                    }
                }
            }
        }
        Ok(task_config)
    }
}

impl Default for TaskConfig {
    fn default() -> Self {
        let mut map = HashMap::new();

        // let press_esc = Task::PressEsc;
        // let press_home = Task::PressHome;
        // let click = Task::Click(0, 0);
        // let swipe = Task::Swipe((0, 0), (200, 0));

        let press_esc = BuiltinTask::ActionPressEsc;
        let press_home = BuiltinTask::ActionPressHome;
        let click = BuiltinTask::ActionClick(0, 0);
        let swipe = BuiltinTask::ActionSwipe((0, 0), (200, 0));
        let click_match = BuiltinTask::ActionClickMatch(MatchTask::Template(
            "ButtonToggleTopNavigator.png".to_string(),
        ));

        map.insert("press_esc".to_string(), press_esc.clone());
        map.insert("press_home".to_string(), press_home.clone());
        map.insert("click_origin".to_string(), click.clone());
        map.insert("swipe_right".to_string(), swipe.clone());
        map.insert("toggle_top_navigator".to_string(), click_match.clone());
        map.insert(
            "multiple".to_string(),
            BuiltinTask::Multi(vec![
                TaskRef::ByInternal(press_esc),
                TaskRef::ByInternal(press_home),
                TaskRef::ByInternal(click),
                TaskRef::ByInternal(swipe),
                TaskRef::ByName("task_name".to_string()),
            ]),
        );

        Self(map)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BuiltinTask {
    Multi(Vec<TaskRef>),
    // Action
    ActionPressEsc,
    ActionPressHome,
    ActionClick(u32, u32),
    ActionSwipe((u32, u32), (u32, u32)),
    ActionClickMatch(MatchTask),
    // Navigate
    NavigateIn(String),
    NavigateOut(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum TaskRef {
    ByInternal(BuiltinTask),
    ByName(String),
}
