use aah_controller::Controller;
use serde::{Deserialize, Serialize};

use crate::{android, Core, TaskRecipe};

use super::AahCore;

pub mod battle;
pub mod choose_level;
pub mod copilot;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActionSet {
    /// General android actions
    Genral(android::actions::ActionSet),
    // BattleDeploy(battle::Deploy),
    // BattleRetreat(battle::Retreat),
    // BattleUseSkill(battle::UseSkill),
    // Copilot(copilot::Copilot),
    // ChooseLevel(choose_level::ChooseLevel),
}

impl From<android::actions::ActionSet> for ActionSet {
    fn from(action: android::actions::ActionSet) -> Self {
        Self::Genral(action)
    }
}

impl TaskRecipe<AahCore> for ActionSet {
    type Res = ();
    fn run(&self, runner: &AahCore) -> anyhow::Result<Self::Res> {
        match self {
            ActionSet::Genral(action) => action.run(runner),
            // ActionSet::Copilot(copilot) => copilot.run(runner),
        }
    }
}
