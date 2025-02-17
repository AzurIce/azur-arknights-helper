use serde::{Deserialize, Serialize};

use crate::{android, task::Runnable};

use super::Aah;

pub mod battle;
pub mod copilot;
pub mod choose_level;

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

impl Runnable<Aah> for ActionSet {
    type Res = ();
    fn run(&self, runner: &Aah) -> anyhow::Result<Self::Res> {
        match self {
            ActionSet::Genral(action) => action.run(runner),
            // ActionSet::Copilot(copilot) => copilot.run(runner),
        }
    }
}