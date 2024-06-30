use image::math::Rect;

use crate::{vision::utils::average_hsv_v, AAH};

use super::{multi_match::MultiMatchAnalyzer, Analyzer};

#[allow(unused)]
#[derive(Debug)]
/// 部署卡片
/// 
/// - `rect`: 位置信息
/// - `available`: 是否可用
pub struct DeployCard {
    rect: Rect,
    available: bool,
}

#[allow(unused)]
#[derive(Debug)]
/// [`DeployAnalyzer`] 的输出
/// 
/// - `deploy_card`: 所有部署卡片信息
pub struct DeployAnalyzerOutput {
    deploy_cards: Vec<DeployCard>,
}

pub struct DeployAnalyzer;

impl Analyzer for DeployAnalyzer {
    type Output = DeployAnalyzerOutput;
    fn analyze(&mut self, core: &AAH) -> Result<Self::Output, String> {
        // Make sure that we are in the operation-start page
        let res =
            MultiMatchAnalyzer::new("battle_deploy-card-cost1.png".to_string(), None, None)
                .analyze(core)?;

        let deploy_cards = res.rects.into_iter().map(|rect| {
            let cropped = res.screen.crop_imm(rect.x, rect.y, rect.width, rect.height);
            let avg_hsv_v = average_hsv_v(&cropped);
            let available = avg_hsv_v > 100;

            let rect = Rect {
                x: rect.x - 45,
                y: rect.y + 6,
                width: 75,
                height: 120,
            };

            DeployCard { rect, available }
        }).collect();

        Ok(DeployAnalyzerOutput {
            deploy_cards
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{vision::analyzer::Analyzer, AAH};

    #[test]
    fn test_deploy_analyzer() {
        let mut core = AAH::connect("127.0.0.1:16384", "../../resources").unwrap();
        let mut analyzer = super::DeployAnalyzer {};
        let output = analyzer.analyze(&mut core).unwrap();
        println!("{:?}", output);
    }
}
