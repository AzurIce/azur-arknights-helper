pub mod manifest;

use std::{fmt::Debug, ops::Deref, path::{Path, PathBuf}, collections::HashMap};
use anyhow::Context;

use manifest::copilot::CopilotConfig;
use crate::actions::Task;

use super::{actions::copilot::Copilot, Action};

// Define Load trait locally
pub trait Load: Sized {
    fn load(path: impl AsRef<Path>) -> anyhow::Result<Self>;
}

// Define generic Resource struct
pub struct Resource<A> {
    pub root: PathBuf,
    pub tasks: HashMap<String, Task<A>>,
}

impl<A> Resource<A> 
where A: serde::de::DeserializeOwned 
{
    pub fn load(root: impl AsRef<Path>) -> anyhow::Result<Self> {
        let root = root.as_ref().to_path_buf();
        // Placeholder for loading tasks. 
        // In a real scenario, we'd scan the 'tasks' directory.
        // For now, initializing empty map to satisfy type check.
        // Real implementation should traverse `root/tasks` and deserialize files.
        let tasks = HashMap::new(); 
        
        Ok(Self {
            root,
            tasks,
        })
    }

    pub fn get_task(&self, name: &str) -> Option<&Task<A>> {
        self.tasks.get(name)
    }
    
    pub fn get_template(&self, path: impl AsRef<Path>) -> anyhow::Result<image::DynamicImage> {
        let full_path = self.root.join("templates").join(path);
        image::open(&full_path).with_context(|| format!("failed to load template at {:?}", full_path))
    }
}


// MARK: AahResource

pub struct AahResource {
    pub inner: Resource<Action>,
    pub copilot_config: CopilotConfig,
}

impl Debug for AahResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AahResource")
    }
}

// MARK: Trait impls

impl Deref for AahResource {
    type Target = Resource<Action>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Load for AahResource {
    fn load(root: impl AsRef<Path>) -> anyhow::Result<Self> {
        let root = root.as_ref();
        let inner = Resource::load(root)?;

        let copilot_config = CopilotConfig::load(root.join("copilot"))?;
        
        Ok(Self {
            inner,
            copilot_config,
        })
    }
}

// MARK: impl AahResource

impl AahResource {
    pub fn get_copilot(&self, name: impl AsRef<str>) -> Option<&Copilot> {
        let name = name.as_ref();
        self.copilot_config.get(name)
    }
}