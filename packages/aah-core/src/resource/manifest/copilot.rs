use std::{collections::HashMap, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::copilot::{Copilot, CopilotAction, CopilotStep, Direction};

#[derive(Serialize, Deserialize, Debug)]
pub struct CopilotConfig(pub HashMap<String, Copilot>);
impl CopilotConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
        let path = path.as_ref();
        let mut config = CopilotConfig(HashMap::new());

        if let Ok(read_dir) = fs::read_dir(path) {
            for entry in read_dir {
                let entry = entry.unwrap();
                if entry.path().extension().and_then(|s| s.to_str()) != Some("toml") {
                    continue;
                }
                if let Ok(task) = fs::read_to_string(entry.path()) {
                    let task = toml::from_str::<Copilot>(&task)?;

                    config.0.insert(task.name.clone(), task);
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
