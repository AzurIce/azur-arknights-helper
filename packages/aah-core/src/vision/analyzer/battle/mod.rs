//! analyzers for battle

pub mod deploy;

use std::time::Instant;

use color_print::{cformat, cprintln};

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
        let log_tag = cformat!("<strong>[BattleAnalyzer]: </strong>");
        cprintln!("{log_tag}analyzing battle...");
        let t = Instant::now();

        // Update cache
        let _ = aah.screen_cap_and_cache().unwrap();
        cprintln!("{log_tag}screen_cap_and_cache cost: {:?}", t.elapsed());
        // Update battle_state
        for (img, state) in [
            (Some("battle_pause.png"), BattleState::Resumed),
            (Some("battle_resume.png"), BattleState::Paused),
            (None, BattleState::Unknown),
        ] {
            match img {
                None => {
                    // battle completed
                    if self.battle_state == BattleState::Resumed {
                        self.start_time = Instant::now();
                        self.battle_state = BattleState::Completed;
                    } else {
                        self.battle_state = state;
                    }
                }
                Some(img) => {
                    let output = SingleMatchAnalyzer::new(img.to_string())
                        .roi((0.875, 0.0), (1.0, 0.125))
                        .use_cache()
                        .analyze(aah)?;
                    if output.res.rect.is_some() {
                        // battle started
                        if self.battle_state == BattleState::Unknown
                            && state != BattleState::Unknown
                        {
                            self.start_time = Instant::now();
                        }
                        self.battle_state = state;
                        break;
                    }
                }
            }
        }

        cprintln!("{log_tag}total cost: {:?}...", t.elapsed());
        Ok(BattleAnalyzerOutput {})
    }
}

#[cfg(test)]
mod test {
    use crate::vision::analyzer::Analyzer;

    use super::BattleAnalyzer;

    #[test]
    fn test_battle_analyzer() {
        let aah = crate::AAH::connect("127.0.0.1:16384", "../../resources", |_| {}).unwrap();
        let mut analyzer = BattleAnalyzer::new();
        let res = analyzer.analyze(&aah).unwrap();
    }
}
