use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use std::path::{Path, PathBuf};
use std::{collections::HashMap, error::Error, fs};

use super::Action;
use super::MatchTask;

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    /// Task 的名称
    pub name: String,
    /// Task 的描述
    pub desc: Option<String>,
    /// Task 的步骤
    pub steps: Vec<TaskStep>,
}

impl Task {
    pub fn from_steps(steps: Vec<TaskStep>) -> Self {
        Self {
            name: "unnamed".to_string(),
            desc: None,
            steps,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_desc(mut self, desc: &str) -> Self {
        self.desc = Some(desc.to_string());
        self
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskStep {
    /// 在此 Step 开始前的延迟
    pub delay_sec: Option<f32>,
    /// 如果此 Step 失败，是否跳过（否则会直接中断退出）
    pub skip_if_failed: Option<bool>,
    /// 重复次数
    pub repeat: Option<u32>,
    /// 每次重试次数
    pub retry: Option<i32>,
    /// 在此 Step 中要执行的 Action
    pub action: Action,
}

impl TaskStep {
    pub fn action(task: Action) -> Self {
        Self {
            delay_sec: None,
            skip_if_failed: None,
            repeat: None,
            retry: None,
            action: task,
        }
    }

    pub fn deplay_sec_f32(mut self, sec: f32) -> Self {
        self.delay_sec = Some(sec);
        self
    }

    pub fn skip_if_failed(mut self) -> Self {
        self.skip_if_failed = Some(true);
        self
    }

    pub fn repeat(mut self, times: u32) -> Self {
        self.repeat = Some(times);
        self
    }

    pub fn retry(mut self, times: i32) -> Self {
        self.retry = Some(times);
        self
    }
}

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
        let task_config = path.join("tasks.toml");
        println!("{:?}", task_config);
        let task_config = fs::read_to_string(task_config)?;
        let mut task_config = toml::from_str::<TaskConfig>(&task_config)?;

        for task_file in get_task_files(path.join("tasks")) {
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

fn startup_task() -> Task {
    Task {
        name: "start_up".to_string(),
        desc: Some("start up to the main screen".to_string()),
        steps: vec![
            TaskStep::action(Action::ActionClickMatch {
                match_task: MatchTask::Template("start_start.png".to_string()),
            })
            .retry(-1),
            TaskStep::action(Action::ActionClickMatch {
                match_task: MatchTask::Template("wakeup_wakeup.png".to_string()),
            })
            .retry(-1),
            TaskStep::action(Action::ActionClickMatch {
                match_task: MatchTask::Template("confirm.png".to_string()),
            })
            .deplay_sec_f32(6.0)
            .retry(3)
            .skip_if_failed(),
            TaskStep::action(Action::ActionClickMatch {
                match_task: MatchTask::Template("qiandao_close.png".to_string()),
            })
            .deplay_sec_f32(2.0)
            .retry(2)
            .skip_if_failed(),
            TaskStep::action(Action::ActionClickMatch {
                match_task: MatchTask::Template("notice_close.png".to_string()),
            })
            .deplay_sec_f32(2.0)
            .retry(2)
            .skip_if_failed(),
        ],
    }
}

fn award_task() -> Task {
    Task {
        name: "award".to_string(),
        desc: None,
        steps: vec![
            TaskStep::action(Action::NavigateIn("mission".to_string())),
            TaskStep::action(Action::ActionClickMatch {
                match_task: MatchTask::Template("mission-week_collect-all.png".to_string()),
            })
            .deplay_sec_f32(0.5)
            .retry(1)
            .skip_if_failed(),
            TaskStep::action(Action::ActionClickMatch {
                match_task: MatchTask::Template("confirm.png".to_string()),
            })
            .deplay_sec_f32(0.5)
            .retry(1)
            .skip_if_failed(),
            TaskStep::action(Action::ActionClickMatch {
                match_task: MatchTask::Template("mission-day_week.png".to_string()),
            })
            .deplay_sec_f32(0.5)
            .retry(1),
            TaskStep::action(Action::ActionClickMatch {
                match_task: MatchTask::Template("mission-week_collect-all.png".to_string()),
            })
            .deplay_sec_f32(0.5)
            .retry(1)
            .skip_if_failed(),
            TaskStep::action(Action::ActionClickMatch {
                match_task: MatchTask::Template("confirm.png".to_string()),
            })
            .deplay_sec_f32(0.5)
            .retry(1)
            .skip_if_failed(),
            TaskStep::action(Action::NavigateOut("mission".to_string())),
        ],
    }
}

pub fn default_tasks() -> Vec<Task> {
    vec![
        Task {
            name: "press_esc".to_string(),
            desc: None,
            steps: vec![TaskStep::action(Action::ActionPressEsc)],
        },
        Task {
            name: "press_home".to_string(),
            desc: None,
            steps: vec![TaskStep::action(Action::ActionPressHome)],
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
