
pub mod manifest;

use std::{fmt::Debug, ops::Deref, path::Path};

use auto_play::resource::{Load, Resource};
use manifest::copilot::CopilotConfig;


use super::{actions::copilot::Copilot, Action};

// MARK: AahResource

pub struct AahResource {
    inner: Resource<Action>,
    copilot_config: CopilotConfig,
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
