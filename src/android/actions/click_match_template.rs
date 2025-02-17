use std::fmt::Debug;

use aah_controller::Controller;
use serde::{Deserialize, Serialize};

use crate::{
    task::Runnable, vision::analyzer::single_match::SingleMatchAnalyzer, CachedScreenCapper,
    GetTemplate,
};

use super::AndroidActionSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickMatchTemplate {
    template: String,
}

impl Into<AndroidActionSet> for ClickMatchTemplate {
    fn into(self) -> AndroidActionSet {
        AndroidActionSet::ClickMatchTemplate(self)
    }
}

impl ClickMatchTemplate {
    pub fn new(template: impl AsRef<str>) -> Self {
        Self {
            template: template.as_ref().to_string(),
        }
    }
}

// TODO: create a new trait like Controller
impl<T: Controller + CachedScreenCapper + GetTemplate> Runnable<T> for ClickMatchTemplate {
    type Res = ();
    fn run(&self, runner: &T) -> anyhow::Result<Self::Res> {
        let template = runner.get_template(&self.template)?;
        let analyzer = SingleMatchAnalyzer::new(template);
        let output = analyzer
            .run(runner)
            .map_err(|err| anyhow::anyhow!("failed to analyze: {err}"))?;
        let rect = output
            .res
            .rect
            .ok_or(anyhow::anyhow!("failed to match {}", self.template))?
            .into();
        runner
            .click_in_rect(rect)
            .map_err(|err| anyhow::anyhow!("controller error: {:?}", err))?;
        Ok(())
    }
}
