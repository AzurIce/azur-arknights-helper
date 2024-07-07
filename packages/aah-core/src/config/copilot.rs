use std::{collections::HashMap, error::Error, fs, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CopilotTask {
    pub level_code: String,
    pub operators: HashMap<String, String>,
    pub steps: Vec<BattleCommand>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Direction {
    Left,
    Up,
    Right,
    Down,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum BattleCommandTime {
    DeltaSec(f32),
    Asap,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BattleCommand {
    Deploy {
        time: BattleCommandTime,
        operator: String,
        tile: (u32, u32),
        direction: Direction,
    },
    AutoSkill {
        time: BattleCommandTime,
        operator: String,
    },
    StopAutoSkill {
        time: BattleCommandTime,
        operator: String,
    },
    Retreat {
        time: BattleCommandTime,
        operator: String,
    },
}

impl BattleCommand {
    pub fn time(&self) -> BattleCommandTime {
        match self {
            BattleCommand::Deploy { time, .. } => time.clone(),
            BattleCommand::AutoSkill { time, .. } => time.clone(),
            BattleCommand::StopAutoSkill { time, .. } => time.clone(),
            BattleCommand::Retreat { time, .. } => time.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CopilotConfig(pub HashMap<String, CopilotTask>);
impl CopilotConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
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
                    let task = toml::from_str::<CopilotTask>(&task)?;

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

    pub fn get_task<S: AsRef<str>>(&self, name: S) -> Result<CopilotTask, String> {
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

pub fn example_copilot_task() -> CopilotTask {
    CopilotTask {
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
            BattleCommand::Deploy {
                time: BattleCommandTime::Asap,
                operator: "myrtle".to_string(),
                tile: (2, 1),
                direction: Direction::Right,
            },
            BattleCommand::AutoSkill {
                time: BattleCommandTime::Asap,
                operator: "myrtle".to_string(),
            },
            BattleCommand::Deploy {
                time: BattleCommandTime::Asap,
                operator: "yato".to_string(),
                tile: (3, 1),
                direction: Direction::Right,
            },
            BattleCommand::Deploy {
                time: BattleCommandTime::Asap,
                operator: "noir_corne".to_string(),
                tile: (4, 1),
                direction: Direction::Right,
            },
            BattleCommand::Deploy {
                time: BattleCommandTime::Asap,
                operator: "melantha".to_string(),
                tile: (2, 2),
                direction: Direction::Down,
            },
            BattleCommand::Deploy {
                time: BattleCommandTime::Asap,
                operator: "ansel".to_string(),
                tile: (5, 2),
                direction: Direction::Up,
            },
            BattleCommand::Deploy {
                time: BattleCommandTime::Asap,
                operator: "rangers".to_string(),
                tile: (1, 3),
                direction: Direction::Down,
            },
            BattleCommand::Deploy {
                time: BattleCommandTime::Asap,
                operator: "durin".to_string(),
                tile: (5, 3),
                direction: Direction::Up,
            },
            BattleCommand::Retreat {
                time: BattleCommandTime::Asap,
                operator: "yato".to_string(),
            },
            BattleCommand::Deploy {
                time: BattleCommandTime::Asap,
                operator: "spot".to_string(),
                tile: (3, 1),
                direction: Direction::Up,
            },
        ],
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::copilot::example_copilot_task;

    #[test]
    fn test_serde() {
        let task = toml::to_string_pretty(&example_copilot_task()).unwrap();
        println!("{}", task);
    }
}
