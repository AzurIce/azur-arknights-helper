use core::fmt;
use std::{
    collections::HashMap,
    error::Error,
    fmt::{Display, Formatter},
    fs,
    path::Path,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Copilot {
    pub name: String,
    pub level_code: String,
    pub operators: HashMap<String, String>,
    pub steps: Vec<CopilotStep>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CopilotStep {
    pub time: CopilotStepTime,
    pub action: CopilotAction,
}

impl CopilotStep {
    pub fn from_action(action: CopilotAction) -> Self {
        Self {
            time: CopilotStepTime::Asap,
            action,
        }
    }

    pub fn with_time(mut self, time: CopilotStepTime) -> Self {
        self.time = time;
        self
    }
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct CopilotTask {
//     pub level_code: String,
//     pub operators: HashMap<String, String>,
//     pub steps: Vec<CopilotAction>,
// }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Direction::Left => "left",
            Direction::Up => "up",
            Direction::Right => "right",
            Direction::Down => "down",
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum CopilotStepTime {
    DeltaSec(f32),
    /// As Soon As Possible
    Asap,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CopilotAction {
    Deploy {
        operator: String,
        position: (u32, u32),
        direction: Direction,
    },
    AutoSkill {
        operator: String,
    },
    StopAutoSkill {
        operator: String,
    },
    Retreat {
        operator: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CopilotConfig(pub HashMap<String, Copilot>);
impl CopilotConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
        let path = path.as_ref();
        let config = path.join("copilots.toml");
        println!("{:?}", config);
        let config = fs::read_to_string(config)?;
        let mut config = toml::from_str::<CopilotConfig>(&config)?;

        if let Ok(read_dir) = fs::read_dir(path.join("copilots")) {
            for entry in read_dir {
                let entry = entry.unwrap();
                if entry.path().extension().and_then(|s| s.to_str()) != Some("toml") {
                    continue;
                }
                if let Ok(task) = fs::read_to_string(entry.path()) {
                    let task = toml::from_str::<Copilot>(&task)?;

                    config.0.insert(
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
        Ok(config)
    }

    pub fn get_task<S: AsRef<str>>(&self, name: S) -> Result<Copilot, String> {
        return self
            .0
            .get(name.as_ref())
            .ok_or("failed to retrive task from copilot_config".to_string())
            .map(|task| task.clone());
    }
}

impl Default for CopilotConfig {
    fn default() -> Self {
        let mut map = HashMap::new();
        let test_tasks = vec![("1-4", example_copilot_task())];
        for (name, task) in test_tasks {
            map.insert(name.to_string(), task);
        }
        Self(map)
    }
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct BattleStep {
//     time: BattleStepTime,
//     action: BattleStepCommand,
// }

pub fn example_copilot_task() -> Copilot {
    Copilot {
        name: "1-4".to_string(),
        level_code: "1-4".to_string(),
        operators: {
            let mut map = HashMap::new();
            map.insert("lancet".to_string(), "char_285_medic2".to_string());
            map.insert("yato".to_string(), "char_502_nblade".to_string());
            map.insert("noir_corne".to_string(), "char_500_noirc".to_string());
            map.insert("rangers".to_string(), "char_503_rang".to_string());
            map.insert("durin".to_string(), "char_501_durin".to_string());
            map.insert("spot".to_string(), "char_284_spot".to_string());
            map.insert("ansel".to_string(), "char_212_ansel".to_string());
            map.insert("melantha".to_string(), "char_208_melan".to_string());
            map.insert("myrtle".to_string(), "char_151_myrtle".to_string());
            map
        },
        steps: vec![
            CopilotStep::from_action(CopilotAction::Deploy {
                operator: "myrtle".to_string(),
                position: (2, 1),
                direction: Direction::Right,
            }),
            CopilotStep::from_action(CopilotAction::AutoSkill {
                operator: "myrtle".to_string(),
            }),
            CopilotStep::from_action(CopilotAction::Deploy {
                operator: "yato".to_string(),
                position: (3, 1),
                direction: Direction::Right,
            }),
            CopilotStep::from_action(CopilotAction::Deploy {
                operator: "noir_corne".to_string(),
                position: (4, 1),
                direction: Direction::Right,
            }),
            CopilotStep::from_action(CopilotAction::Deploy {
                operator: "melantha".to_string(),
                position: (2, 2),
                direction: Direction::Down,
            }),
            CopilotStep::from_action(CopilotAction::Deploy {
                operator: "ansel".to_string(),
                position: (5, 2),
                direction: Direction::Up,
            }),
            CopilotStep::from_action(CopilotAction::Deploy {
                operator: "rangers".to_string(),
                position: (1, 3),
                direction: Direction::Down,
            }),
            CopilotStep::from_action(CopilotAction::Deploy {
                operator: "durin".to_string(),
                position: (5, 3),
                direction: Direction::Up,
            }),
            CopilotStep::from_action(CopilotAction::Retreat {
                operator: "yato".to_string(),
            }),
            CopilotStep::from_action(CopilotAction::Deploy {
                operator: "spot".to_string(),
                position: (3, 1),
                direction: Direction::Up,
            }),
        ],
    }
}

#[cfg(test)]
mod test {
    use super::example_copilot_task;

    #[test]
    fn test_serde() {
        let task = toml::to_string_pretty(&example_copilot_task()).unwrap();
        println!("{}", task);
    }
}
