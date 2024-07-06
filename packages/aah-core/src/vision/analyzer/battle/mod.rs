//! analyzers for battle

pub mod deploy;

use std::time::Instant;

use color_print::{cformat, cprintln};
use deploy::{DeployAnalyzer, DeployCard};
use serde::Serialize;

use super::{single_match::SingleMatchAnalyzer, Analyzer};

#[derive(Debug, Serialize, Clone)]
pub struct BattleAnalyzerOutput {
    battle_state: BattleState,
    deploy_cards: Vec<DeployCard>,
}

pub enum Speed {
    Speed1,
    Speed2,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize)]
pub enum BattleState {
    Unknown,
    Resumed,
    Paused,
    Completed,
}

pub struct BattleAnalyzer {
    pub battle_state: BattleState,
    deploy_cards: Vec<DeployCard>,
    start_time: Instant,
    deploy_analyzer: Option<DeployAnalyzer>,
}

impl BattleAnalyzer {
    pub fn new() -> Self {
        Self {
            battle_state: BattleState::Unknown,
            deploy_cards: Vec::new(),
            start_time: Instant::now(),
            deploy_analyzer: None,
        }
    }
}

impl Analyzer for BattleAnalyzer {
    type Output = BattleAnalyzerOutput;
    fn analyze(&mut self, aah: &crate::AAH) -> Result<Self::Output, String> {
        let log_tag = cformat!("<strong>[BattleAnalyzer]: </strong>");
        cprintln!("{log_tag}analyzing battle...");
        let t = Instant::now();

        if self.deploy_analyzer.is_none() {
            self.deploy_analyzer = Some(
                DeployAnalyzer::new().use_cache().with_opers(
                    vec![
                        "char_285_medic2",
                        "char_502_nblade",
                        "char_500_noirc",
                        "char_503_rang",
                        "char_501_durin",
                        "char_284_spot",
                        "char_212_ansel",
                        "char_208_melan",
                        "char_151_myrtle",
                    ]
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
                    aah,
                ),
            );
        }

        // Update cache
        let _ = aah.screen_cap_and_cache().unwrap();
        cprintln!("{log_tag}screen_cap_and_cache cost: {:?}", t.elapsed());
        // Update battle_state
        for (img, state) in [
            (Some("battle_pause.png"), BattleState::Resumed),
            (Some("battle_pause-dim.png"), BattleState::Resumed),
            (Some("battle_resume.png"), BattleState::Paused),
            (None, BattleState::Unknown),
        ] {
            match img {
                None => {
                    // battle completed
                    if self.battle_state == BattleState::Resumed
                        || self.battle_state == BattleState::Paused
                    {
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

        if self.battle_state != BattleState::Unknown {
            // TODO: Analyze battlefield (deploy)
            let output = self.deploy_analyzer.as_mut().unwrap().analyze(aah)?;
            self.deploy_cards = output.deploy_cards;
        }

        cprintln!("{log_tag}total cost: {:?}...", t.elapsed());
        Ok(BattleAnalyzerOutput {
            battle_state: self.battle_state,
            deploy_cards: self.deploy_cards.clone(),
        })
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
