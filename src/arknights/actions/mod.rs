use serde::{Deserialize, Serialize};

use crate::{android, task::Runnable};

pub mod battle;
pub mod copilot;
pub mod choose_level;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    /// General android actions
    Genral(android::actions::AndroidActionSet),
    BattleDeploy(battle::Deploy),
    BattleRetreat(battle::Retreat),
    BattleUseSkill(battle::UseSkill),
    Copilot(copilot::Copilot),
    ChooseLevel(choose_level::ChooseLevel),
}
