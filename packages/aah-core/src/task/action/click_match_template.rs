use serde::{Deserialize, Serialize};

use crate::{
    task::Runnable,
    vision::analyzer::{single_match::SingleMatchAnalyzer, Analyzer},
    AAH,
};

use super::Action;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickMatchTemplate {
    template: String,
}

impl Into<Action> for ClickMatchTemplate {
    fn into(self) -> Action {
        Action::ClickMatchTemplate(self)
    }
}

impl ClickMatchTemplate {
    pub fn new(template: impl AsRef<str>) -> Self {
        Self {
            template: template.as_ref().to_string(),
        }
    }
}

impl Runnable for ClickMatchTemplate {
    type Err = anyhow::Error;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        let mut analyzer = SingleMatchAnalyzer::new(&aah.resource.root, self.template.clone());
        let output = analyzer
            .analyze(aah)
            .map_err(|err| anyhow::anyhow!("failed to analyze: {err}"))?;
        let rect = output
            .res
            .rect
            .ok_or(anyhow::anyhow!("failed to match {}", self.template))?;
        aah.controller
            .click_in_rect(rect)
            .map_err(|err| anyhow::anyhow!("controller error: {:?}", err))
    }
}
