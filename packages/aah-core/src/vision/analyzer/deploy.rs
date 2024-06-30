use image::{DynamicImage};
use serde::Serialize;

use crate::{
    vision::utils::{average_hsv_v, draw_box, Rect},
    AAH,
};

use super::{multi_match::MultiMatchAnalyzer, Analyzer};

#[allow(unused)]
#[derive(Debug, Serialize)]
/// 部署卡片
///
/// - `rect`: 位置信息
/// - `available`: 是否可用
pub struct DeployCard {
    pub rect: Rect,
    pub available: bool,
}

#[allow(unused)]
#[derive(Debug)]
/// [`DeployAnalyzer`] 的输出
///
/// - `deploy_card`: 所有部署卡片信息
pub struct DeployAnalyzerOutput {
    pub screen: DynamicImage,
    pub deploy_cards: Vec<DeployCard>,
    pub res_screen: DynamicImage,
}

pub struct DeployAnalyzer;

impl Analyzer for DeployAnalyzer {
    type Output = DeployAnalyzerOutput;
    fn analyze(&mut self, core: &AAH) -> Result<Self::Output, String> {
        // Make sure that we are in the operation-start page
        let res = MultiMatchAnalyzer::new("battle_deploy-card-cost1.png".to_string(), None, None)
            .analyze(core)?;

        let deploy_cards: Vec<DeployCard> = res
            .rects
            .into_iter()
            .map(|rect| {
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
            })
            .collect();

        let mut res_screen = res.screen.clone();
        for deploy_card in &deploy_cards {
            let color = if deploy_card.available {
                [0, 255, 0, 255]
            } else {
                [255, 0, 0, 255]
            };
            let rect = deploy_card.rect.clone();

            draw_box(
                &mut res_screen,
                rect.x as i32,
                rect.y as i32,
                rect.width,
                rect.height,
                color,
            );
        }

        Ok(DeployAnalyzerOutput {
            screen: res.screen,
            deploy_cards,
            res_screen,
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
