//! analyzers for battle

use std::time::Instant;

use super::{single_match::SingleMatchAnalyzer, Analyzer};

pub struct BattleAnalyzerOutput {}

pub enum Speed {
    Speed1,
    Speed2,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BattleState {
    Unknown,
    Resumed,
    Paused,
    Completed,
}

pub struct BattleAnalyzer {
    battle_state: BattleState,
    start_time: Instant,
}

impl BattleAnalyzer {
    pub fn new() -> Self {
        Self {
            battle_state: BattleState::Unknown,
            start_time: Instant::now(),
        }
    }
}

impl Analyzer for BattleAnalyzer {
    type Output = BattleAnalyzerOutput;
    fn analyze(&mut self, aah: &crate::AAH) -> Result<Self::Output, String> {
        // Update battle_state
        let output = SingleMatchAnalyzer::new("battle_paused.png".to_string()).analyze(aah)?;
        if output.res.rect.is_some() {
            self.battle_state = BattleState::Resumed;

            // battle started
            if self.battle_state == BattleState::Unknown {
                self.start_time = Instant::now();
            }
        }

        let output = SingleMatchAnalyzer::new("battle_paused.png".to_string()).analyze(aah)?;
        if output.res.rect.is_some() {
            self.battle_state = BattleState::Resumed;

            // battle started
            if self.battle_state == BattleState::Unknown {
                self.start_time = Instant::now();
            }
        }



        Ok(BattleAnalyzerOutput {})
    }
}
