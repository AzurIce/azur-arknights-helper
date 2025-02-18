use std::{ops::Deref, path::Path};

use manifest::copilot::CopilotConfig;

use crate::{
    resource::{GeneralAahResource, GetTask, Load, ResRoot},
    task::Task,
};

pub mod manifest;

use super::{actions::copilot::Copilot, ActionSet};

// MARK: AahResource

pub struct AahResource {
    inner: GeneralAahResource<ActionSet>,
    pub copilot_config: CopilotConfig,
}

// MARK: Trait impls

impl Deref for AahResource {
    type Target = GeneralAahResource<ActionSet>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl GetTask<ActionSet> for AahResource {
    fn get_task(&self, name: impl AsRef<str>) -> Option<&Task<ActionSet>> {
        self.inner.get_task(name)
    }
}

impl ResRoot for AahResource {
    fn res_root(&self) -> &Path {
        self.inner.res_root()
    }
}

impl Load for AahResource {
    fn load(root: impl AsRef<Path>) -> anyhow::Result<Self> {
        let root = root.as_ref();
        let inner = GeneralAahResource::load(root)?;

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
