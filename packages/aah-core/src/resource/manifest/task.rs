use serde::{Deserialize, Serialize};

use std::path::{Path, PathBuf};
use std::{collections::HashMap, fs};

use crate::task::action::{
    Action, ClickMatchTemplate, NavigateIn, NavigateOut, PressEsc, PressHome,
};
use crate::task::{Task, TaskStep};

fn get_task_files(path: impl AsRef<Path>) -> Vec<PathBuf> {
    let mut task_files = vec![];
    if let Ok(read_dir) = fs::read_dir(path) {
        for entry in read_dir {
            let entry = entry.unwrap();
            let file_type = entry.file_type().unwrap();
            if file_type.is_dir() {
                task_files.extend(get_task_files(entry.path()));
            } else if file_type.is_file() {
                task_files.push(entry.path());
            }
        }
    }
    task_files
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskConfig(pub HashMap<String, Task>);
impl TaskConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
        let path = path.as_ref();
        let mut task_config = TaskConfig(HashMap::new());
        for task_file in get_task_files(path) {
            if let Ok(task) = fs::read_to_string(task_file) {
                let task = toml::from_str::<Task>(&task)?;

                task_config.0.insert(task.name.to_string(), task);
            }
        }
        Ok(task_config)
    }

    pub fn get_task<S: AsRef<str>>(&self, name: S) -> Result<&Task, String> {
        return self
            .0
            .get(name.as_ref())
            .ok_or("failed to retrive task from task_config".to_string());
    }
}

impl Default for TaskConfig {
    fn default() -> Self {
        let mut map = HashMap::new();
        let test_tasks = default_tasks();
        for task in test_tasks {
            map.insert(task.name.clone(), task);
        }
        Self(map)
    }
}

#[allow(unused)]
fn startup_task() -> Task {
    Task {
        name: "start_up".to_string(),
        desc: Some("start up to the main screen".to_string()),
        steps: vec![
            TaskStep::from_action(ClickMatchTemplate::new("start_start.png")).with_retry(-1),
            TaskStep::from_action(ClickMatchTemplate::new("wakeup_wakeup.png")).with_retry(-1),
            TaskStep::from_action(ClickMatchTemplate::new("confirm.png"))
                .with_delay(6.0)
                .with_retry(3)
                .skip_if_failed(),
            TaskStep::from_action(ClickMatchTemplate::new("qiandao_close.png"))
                .with_delay(2.0)
                .with_retry(2)
                .skip_if_failed(),
            TaskStep::from_action(ClickMatchTemplate::new("notice_close.png"))
                .with_delay(2.0)
                .with_retry(2)
                .skip_if_failed(),
        ],
    }
}

#[allow(unused)]
fn award_task() -> Task {
    Task {
        name: "award".to_string(),
        desc: None,
        steps: vec![
            TaskStep::from_action(NavigateIn::new("mission")),
            TaskStep::from_action(ClickMatchTemplate::new("mission-week_collect-all.png"))
                .with_delay(0.5)
                .with_retry(1)
                .skip_if_failed(),
            TaskStep::from_action(ClickMatchTemplate::new("confirm.png"))
                .with_delay(0.5)
                .with_retry(1)
                .skip_if_failed(),
            TaskStep::from_action(ClickMatchTemplate::new("mission-day_week.png"))
                .with_delay(0.5)
                .with_retry(1),
            TaskStep::from_action(ClickMatchTemplate::new("mission-week_collect-all.png"))
                .with_delay(0.5)
                .with_retry(1)
                .skip_if_failed(),
            TaskStep::from_action(ClickMatchTemplate::new("confirm.png"))
                .with_delay(0.5)
                .with_retry(1)
                .skip_if_failed(),
            TaskStep::from_action(NavigateOut::new("mission")),
        ],
    }
}

pub fn default_tasks() -> Vec<Task> {
    vec![
        Task {
            name: "press_esc".to_string(),
            desc: None,
            steps: vec![TaskStep::from_action(PressEsc)],
        },
        Task {
            name: "press_home".to_string(),
            desc: None,
            steps: vec![TaskStep::from_action(PressHome)],
        },
    ]
}

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
    fn test_ser_task() {
        // let task = startup_task();
        let task = award_task();
        let config = toml::to_string_pretty(&task).unwrap();
        println!("{}", config);
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
}
