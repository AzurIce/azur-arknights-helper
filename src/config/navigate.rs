use std::{collections::HashMap, error::Error, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::task::MatchTask;

use super::task::{BuiltinTask, TaskRef};

#[cfg(test)]
mod test {
    use std::{
        error::Error,
        fs::{self, File, OpenOptions},
        io::Write,
    };

    use crate::config;

    use super::*;

    #[test]
    fn test_navigate_config() -> Result<(), Box<dyn Error>> {
        let navigate_config = NavigateConfig::default();
        let config_str = toml::to_string_pretty(&navigate_config)?;

        let config_file = "./resources/navigates.toml";
        let mut open_options = OpenOptions::new();
        open_options.write(true).create(true);

        let mut file = open_options.open(config_file)?;
        file.write_fmt(format_args!("{}", config_str))?;
        Ok(())
    }
    
    #[test]
    fn test_load_navigate_config() -> Result<(), Box<dyn Error>> {
        let config = NavigateConfig::load()?;
        println!("{:?}", config);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NavigateConfig(pub HashMap<String, Navigate>);
impl NavigateConfig {
    pub fn load() -> Result<NavigateConfig, Box<dyn Error>> {
        let config = fs::read_to_string("./resources/navigates.toml")?;
        let mut config = toml::from_str::<NavigateConfig>(&config)?;

        if let Ok(read_dir) = fs::read_dir(PathBuf::from("./resources/navigates/")) {
            for entry in read_dir {
                let entry = entry.unwrap();
                if entry.path().extension().and_then(|s| s.to_str()) != Some("toml") {
                    continue;
                }
                if let Ok(navigate) = fs::read_to_string(entry.path()) {
                    if let Ok(navigate) = toml::from_str::<Navigate>(&navigate) {
                        config.0.insert(
                            entry.path().file_prefix().and_then(|s| s.to_str()).unwrap().to_string(),
                            navigate,
                        );
                    }
                }
            }
        }
        Ok(config)
    }
}

impl Default for NavigateConfig {
    fn default() -> Self {
        let mut map = HashMap::new();

        map.insert(
            "base".to_string(),
            Navigate {
                enter_task: TaskRef::ByInternal(BuiltinTask::ActionClickMatch(MatchTask::Template(
                    "EnterInfrastMistCity.png".to_string(),
                ))),
                exit_task: TaskRef::ByName("back".to_string()),
            },
        );

        map.insert(
            "mission".to_string(),
            Navigate {
                enter_task: TaskRef::ByInternal(BuiltinTask::ActionClickMatch(MatchTask::Template(
                    "EnterMissionMistCity.png".to_string(),
                ))),
                exit_task: TaskRef::ByName("back".to_string()),
            },
        );

        Self(map)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Navigate {
    pub enter_task: TaskRef,
    pub exit_task: TaskRef,
}
