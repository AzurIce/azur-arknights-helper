//! analyzers for battle

pub mod deploy;

use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use auto_play::actions::Runnable;
use color_print::{cformat, cprintln};
use deploy::{DeployAnalyzer, DeployCard};
use image::DynamicImage;
use serde::Serialize;

use crate::{
    vision::analyzer::{matching::MatchOptions, single_match::SingleMatchAnalyzer},
    AahCore,
};

#[derive(Debug, Serialize, Clone)]
/// [`BattleAnalyzer`] 的分析结果：
/// - `battle_state`: 当前关卡的战斗状态，见 [`BattleState`]
///   依据右上角的 暂停/继续 按钮来判断：
///   - 找到按钮前为 [`BattleState::Unknown`]
///   - 之后为 [`BattleState::Resumed`] 或 [`BattleState::Paused`]
///   - 按钮丢失后为 [`BattleState::Completed`]
/// - `deploy_cards`: 部署卡片列表，包含了部署卡片的干员、位置、就绪信息，见 [`DeployCard`]
pub struct BattleAnalyzerOutput {
    pub battle_state: BattleState,
    pub deploy_cards: Vec<DeployCard>,
}

// pub enum Speed {
//     Speed1,
//     Speed2,
// }

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize)]
/// 关卡的战斗状态，判断依据见 [`BattleAnalyzerOutput`]
pub enum BattleState {
    Unknown,
    Resumed,
    Paused,
    Completed,
}

/// 战场分析器，详情见输出分析结果 [`BattleAnalyzer`]
///
/// # Example
/// ```rust
/// use aah_core::vision::analyzer::battle::BattleAnalyzer;
/// use image;
///
/// fn main() {
///     let analyzer = BattleAnalyzer::new(
///         "../../resources",
///         vec![
///             "char_1028_texas2",
///             "char_4087_ines",
///             "char_479_sleach",
///             "char_222_bpipe",
///             "char_1016_agoat2",
///             "char_245_cello",
///             "char_1020_reed2",
///             "char_4117_ray",
///             "char_2025_shu",
///             "char_1032_excu2",
///             "char_1035_wisdel",
///             "char_311_mudrok",
///         ],
///     );
///     let image = image::open("../../resources/templates/MUMU-1920x1080/1-4.png").unwrap();
///     let output = analyzer.analyze_image(&image).unwrap();
///     println!("{:?}", output.deploy_cards);
/// }
/// ```
pub struct BattleAnalyzer {
    res_dir: PathBuf,
    deploy_analyzer: DeployAnalyzer,
}

impl BattleAnalyzer {
    /// 创建一个新的 [`BattleAnalyzer`]
    ///
    /// - `res_dir`: 资源文件路径，会通过 [`crate::utils::resource::get_template`] 加载模板
    /// - `oper_names`: 内部的 [`DeployAnalyzer`] 识别的干员，为游戏内部资源文件的命名方式，如 `char_102_texas`, `char_1028_texas2`
    pub fn new<P: AsRef<Path>, S: AsRef<str>>(res_dir: P, oper_names: &[S]) -> Self {
        let deploy_analyzer = DeployAnalyzer::new(&res_dir, oper_names);
        Self {
            res_dir: res_dir.as_ref().to_path_buf(),
            deploy_analyzer,
        }
    }

    /// 分析传入的 `image`
    pub fn analyze_image(&self, image: &DynamicImage) -> anyhow::Result<BattleAnalyzerOutput> {
        let log_tag = cformat!("<strong>[BattleAnalyzer]: </strong>");
        cprintln!("{log_tag}analyzing battle...");
        let t = Instant::now();

        // Determine battle_state
        let mut battle_state = BattleState::Unknown;
        for (img, state) in [
            ("battle_resume.png", BattleState::Paused),
            ("battle_pause.png", BattleState::Resumed),
            ("battle_pause-dim.png", BattleState::Resumed),
            ("battle_pause-dim-dim.png", BattleState::Resumed),
        ] {
            let output = SingleMatchAnalyzer::new(&self.res_dir, img.to_string())
                .with_options(
                    MatchOptions::default()
                        .with_roi((0.875, 0.0), (1.0, 0.125))
                        .use_cache(),
                )
                .analyze_image(image)?;
            if output.res.result.is_some() {
                battle_state = state;
                break;
            }
        }

        let deploy_cards = if battle_state != BattleState::Unknown {
            // TODO: Analyze battlefield (deploy)
            let output = self.deploy_analyzer.analyze_image(image)?;
            output.deploy_cards
        } else {
            Vec::new()
        };

        cprintln!("{log_tag}cost: {:?}...", t.elapsed());
        Ok(BattleAnalyzerOutput {
            battle_state,
            deploy_cards,
        })
    }
}

impl Runnable<AahCore> for BattleAnalyzer {
    type Output = BattleAnalyzerOutput;
    fn execute(&self, executor: &AahCore) -> anyhow::Result<Self::Output> {
        let screen = executor.screen_cap_and_cache().unwrap();
        self.analyze_image(&screen)
    }
}

#[cfg(test)]
mod test {
    use super::BattleAnalyzer;

    #[test]
    fn test_battle_analyzer() {
        let analyzer = BattleAnalyzer::new(
            "../../resources",
            &[
                "char_1028_texas2",
                "char_4087_ines",
                "char_479_sleach",
                "char_222_bpipe",
                "char_1016_agoat2",
                "char_245_cello",
                "char_1020_reed2",
                "char_4117_ray",
                "char_2025_shu",
                "char_1032_excu2",
                "char_1035_wisdel",
                "char_311_mudrok",
            ],
        );
        let image = image::open("../../resources/templates/MUMU-1920x1080/1-4.png").unwrap();
        let output = analyzer.analyze_image(&image).unwrap();
        println!("{:?}", output.deploy_cards);
    }
}
