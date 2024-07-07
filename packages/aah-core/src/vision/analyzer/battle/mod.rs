//! analyzers for battle

pub mod deploy;

use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use color_print::{cformat, cprintln};
use deploy::{DeployAnalyzer, DeployCard, EXAMPLE_DEPLOY_OPERS};
use image::DynamicImage;
use serde::Serialize;

use super::{single_match::SingleMatchAnalyzer, Analyzer};

#[derive(Debug, Serialize, Clone)]
/// [`BattleAnalyzer`] 的分析结果：
/// - `battle_state`: 战斗状态，见 [`BattleState`]
/// - `deploy_cards`: 部署卡片列表，见 [`DeployCard`]
pub struct BattleAnalyzerOutput {
    pub battle_state: BattleState,
    pub deploy_cards: Vec<DeployCard>,
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
    res_dir: PathBuf,
    pub battle_state: BattleState,
    deploy_cards: Vec<DeployCard>,
    start_time: Instant,
    deploy_analyzer: DeployAnalyzer,
}

impl BattleAnalyzer {
    /// 创建一个新的 [`BattleAnalyzer`]
    ///
    ///
    pub fn new<P: AsRef<Path>>(res_dir: P) -> Self {
        let deploy_analyzer = DeployAnalyzer::new(&res_dir, EXAMPLE_DEPLOY_OPERS.to_vec());
        Self {
            res_dir: res_dir.as_ref().to_path_buf(),
            battle_state: BattleState::Unknown,
            deploy_cards: Vec::new(),
            start_time: Instant::now(),
            deploy_analyzer,
        }
    }

    fn analyze_image(&mut self, image: &DynamicImage) -> Result<BattleAnalyzerOutput, String> {
        let log_tag = cformat!("<strong>[BattleAnalyzer]: </strong>");
        cprintln!("{log_tag}analyzing battle...");
        let t = Instant::now();

        // Update cache
        cprintln!("{log_tag}screen_cap_and_cache cost: {:?}", t.elapsed());
        // Update battle_state
        for (img, state) in [
            (Some("battle_resume.png"), BattleState::Paused),
            (Some("battle_pause.png"), BattleState::Resumed),
            (Some("battle_pause-dim.png"), BattleState::Resumed),
            (Some("battle_pause-dim-dim.png"), BattleState::Resumed),
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
                    let output = SingleMatchAnalyzer::new(&self.res_dir, img.to_string())
                        .roi((0.875, 0.0), (1.0, 0.125))
                        .use_cache()
                        .analyze_image(image)?;
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
            let output = self.deploy_analyzer.analyze_image(image)?;
            self.deploy_cards = output.deploy_cards;
        }

        cprintln!("{log_tag}total cost: {:?}...", t.elapsed());
        Ok(BattleAnalyzerOutput {
            battle_state: self.battle_state,
            deploy_cards: self.deploy_cards.clone(),
        })
    }
}

impl Analyzer for BattleAnalyzer {
    type Output = BattleAnalyzerOutput;
    fn analyze(&mut self, aah: &crate::AAH) -> Result<Self::Output, String> {
        let screen = aah.screen_cap_and_cache().unwrap();
        self.analyze_image(&screen)
    }
}

#[cfg(test)]
mod test {
    use crate::vision::analyzer::Analyzer;

    use super::BattleAnalyzer;

    #[test]
    fn test_battle_analyzer() {
        let aah = crate::AAH::connect("127.0.0.1:16384", "../../resources", |_| {}).unwrap();
        let mut analyzer = BattleAnalyzer::new(&aah.res_dir);
        let res = analyzer.analyze(&aah).unwrap();
        println!("{:?}", res)
    }
}
