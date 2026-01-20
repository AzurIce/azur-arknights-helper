pub mod manifest;

use std::{collections::HashMap, fmt::Debug, path::{Path, PathBuf}};
use anyhow::Context;

use manifest::copilot::CopilotConfig;
use crate::actions::{Action, Task};

use super::actions::copilot::Copilot;

// Define Load trait locally
pub trait Load: Sized {
    fn load(path: impl AsRef<Path>) -> anyhow::Result<Self>;
}

// MARK: AahResource

pub struct AahResource {
    pub root: PathBuf,
    pub tasks: HashMap<String, Task<Action>>,
    pub copilot_config: CopilotConfig,
}

impl Debug for AahResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AahResource")
    }
}

// MARK: Trait impls

impl Load for AahResource {
    fn load(root: impl AsRef<Path>) -> anyhow::Result<Self> {
        let root = root.as_ref().to_path_buf();
        
        // Placeholder for loading tasks. 
        // In a real scenario, we'd scan the 'tasks' directory.
        let tasks = HashMap::new(); 

        let copilot_config = CopilotConfig::load(root.join("copilot"))?;
        
        Ok(Self {
            root,
            tasks,
            copilot_config,
        })
    }
}

// MARK: impl AahResource

impl AahResource {
    pub fn get_task(&self, name: &str) -> Option<&Task<Action>> {
        self.tasks.get(name)
    }
    
    pub fn get_template(&self, path: impl AsRef<Path>) -> anyhow::Result<image::DynamicImage> {
        let full_path = self.root.join("templates").join(path);
        image::open(&full_path).with_context(|| format!("failed to load template at {:?}", full_path))
    }

    pub fn get_copilot(&self, name: impl AsRef<str>) -> Option<&Copilot> {
        let name = name.as_ref();
        self.copilot_config.get(name)
    }
}
