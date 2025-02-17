use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

use image::DynamicImage;
use manifest::copilot::CopilotConfig;

use crate::{
    resource::{GeneralAahResource, Load},
    utils::LazyImage,
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
    pub fn get_copilot(&self, name: &str) -> Option<&Copilot> {
        self.copilot_config.get(name)
    }
}
