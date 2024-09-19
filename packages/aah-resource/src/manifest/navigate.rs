use std::{collections::HashMap, fs, path::Path};

use serde::{Deserialize, Serialize};

use super::Action;

#[derive(Serialize, Deserialize, Debug)]
pub struct NavigateConfig(pub HashMap<String, Navigate>);
impl NavigateConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<NavigateConfig, anyhow::Error> {
        let path = path.as_ref();
        let config = path.join("navigates.toml");
        let config = fs::read_to_string(config)?;
        let mut config = toml::from_str::<NavigateConfig>(&config)?;

        if let Ok(read_dir) = fs::read_dir(path.join("navigates")) {
            for entry in read_dir {
                let entry = entry.unwrap();
                if entry.path().extension().and_then(|s| s.to_str()) != Some("toml") {
                    continue;
                }
                if let Ok(navigate) = fs::read_to_string(entry.path()) {
                    if let Ok(navigate) = toml::from_str::<Navigate>(&navigate) {
                        config.0.insert(
                            entry
                                .path()
                                .file_prefix()
                                .and_then(|s| s.to_str())
                                .unwrap()
                                .to_string(),
                            navigate,
                        );
                    }
                }
            }
        }
        Ok(config)
    }
    pub fn get_navigate<S: AsRef<str>>(&self, name: S) -> Result<Navigate, String> {
        self.0
            .get(name.as_ref())
            .ok_or(format!(
                "failed to retrive navigate by name {:?}",
                name.as_ref()
            ))
            .map(|navigate| navigate.clone())
    }
}

impl Default for NavigateConfig {
    fn default() -> Self {
        let mut map = HashMap::new();

        map.insert(
            "base".to_string(),
            Navigate {
                enter: Action::ByName("enter_base".to_string()),
                exit: Action::ByName("back".to_string()),
            },
        );

        map.insert(
            "mission".to_string(),
            Navigate {
                enter: Action::ByName("enter_mission".to_string()),
                exit: Action::ByName("back".to_string()),
            },
        );

        Self(map)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Navigate {
    pub enter: Action,
    pub exit: Action,
}

#[cfg(test)]
mod test {
    use std::{error::Error, fs::OpenOptions, io::Write};

    use super::*;

    #[test]
    fn test_ser_default_navigate() {
        let navigate_config = NavigateConfig::default();
        let config_str = toml::to_string_pretty(&navigate_config).unwrap();
        println!("{}", config_str);
    }

    #[test]
    fn write_default_navigate_config() -> Result<(), Box<dyn Error>> {
        let navigate_config = NavigateConfig::default();
        let config_str = toml::to_string_pretty(&navigate_config)?;

        let config_file = "../../resources/navigates.toml";
        let mut open_options = OpenOptions::new();
        open_options.write(true).create(true);

        let mut file = open_options.open(config_file)?;
        file.write_fmt(format_args!("{}", config_str))?;
        Ok(())
    }

    #[test]
    fn test_load_navigate_config() -> Result<(), Box<dyn Error>> {
        let config = NavigateConfig::load("../../resources")?;
        println!("{:?}", config);
        Ok(())
    }
}
