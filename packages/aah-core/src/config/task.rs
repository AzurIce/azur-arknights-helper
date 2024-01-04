use serde::{Deserialize, Serialize};

use std::path::Path;
use std::{collections::HashMap, error::Error, fs};

use crate::config::task;
use crate::task::wrapper::GenericTaskWrapper;
use crate::task::{match_task::MatchTask, Task};
use crate::AAH;

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

        // let press_esc = Task::PressEsc;
        // let press_home = Task::PressHome;
        // let click = Task::Click(0, 0);
        // let swipe = Task::Swipe((0, 0), (200, 0));

        let press_esc = BuiltinTask::ActionPressEsc(ActionPressEsc::new(None));
        let press_home = BuiltinTask::ActionPressHome(ActionPressHome::new(None));
        let click =
            BuiltinTask::ActionClick(ActionClick::new(0, 0, Some(GenericTaskWrapper::default())));
        let swipe = BuiltinTask::ActionSwipe(ActionSwipe::new((0, 0), (200, 0), 1.0, None));
        let click_match = BuiltinTask::ActionClickMatch(ActionClickMatch::new(
            MatchTask::Template("ButtonToggleTopNavigator.png".to_string()),
            None,
        ));
        let navigate_in = BuiltinTask::NavigateIn("name".to_string());
        let navigate_out = BuiltinTask::NavigateIn("name".to_string());
        let by_name = BuiltinTask::ByName(ByName::new("press_esc", Some(GenericTaskWrapper::default())));

        map.insert("press_esc".to_string(), press_esc.clone());
        map.insert("press_home".to_string(), press_home.clone());
        map.insert("click_origin".to_string(), click.clone());
        map.insert("swipe_right".to_string(), swipe.clone());
        map.insert("toggle_top_navigator".to_string(), click_match.clone());
        map.insert("navigate_in".to_string(), navigate_in.clone());
        map.insert("navigate_out".to_string(), navigate_out.clone());
        map.insert("by_name".to_string(), by_name.clone());
        map.insert(
            "multiple".to_string(),
            BuiltinTask::Multi(Multi::new(
                vec![
                    press_esc,
                    navigate_in,
                    press_home,
                    click,
                    swipe,
                    by_name,
                ],
                false,
                None,
            )),
        );

        Self(map)
    }
}

use crate::task::builtins::*;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BuiltinTask {
    ByName(ByName),
    Multi(Multi),
    // Action
    ActionPressEsc(ActionPressEsc),
    ActionPressHome(ActionPressHome),
    ActionClick(ActionClick),
    ActionSwipe(ActionSwipe),
    ActionClickMatch(ActionClickMatch),
    // Navigate
    NavigateIn(String),
    NavigateOut(String),
}

impl Task for BuiltinTask {
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        match self {
            BuiltinTask::ByName(task) => task.run(aah),
            BuiltinTask::Multi(task) => task.run(aah),
            BuiltinTask::ActionPressEsc(task) => task.run(aah),
            BuiltinTask::ActionPressHome(task) => task.run(aah),
            BuiltinTask::ActionClick(task) => task.run(aah),
            BuiltinTask::ActionSwipe(task) => task.run(aah),
            BuiltinTask::ActionClickMatch(task) => task.run(aah),
            BuiltinTask::NavigateIn(navigate) => Navigate::NavigateIn(navigate.clone()).run(aah),
            BuiltinTask::NavigateOut(navigate) => Navigate::NavigateOut(navigate.clone()).run(aah),
        }
    }
}
