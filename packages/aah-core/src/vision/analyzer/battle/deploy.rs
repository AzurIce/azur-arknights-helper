use std::{path::Path, time::Instant};

use color_print::{cformat, cprintln};
use image::DynamicImage;
use serde::Serialize;

use crate::{
    vision::{
        analyzer::multi_match::MultiMatchAnalyzer,
        matcher::best_matcher::BestMatcher,
        utils::{average_hsv_v, draw_box, resource::get_opers_avatars, Rect},
    },
    AAH,
};

use super::Analyzer;

#[allow(unused)]
#[derive(Debug, Serialize, Clone)]
/// 部署卡片
///
/// - `oper_name`: 干员名
/// - `rect`: 位置信息
/// - `available`: 是否可用
pub struct DeployCard {
    pub oper_name: String,
    pub rect: Rect,
    pub available: bool,
}

#[allow(unused)]
/// [`DeployAnalyzer`] 的输出
///
/// - `screen`: 作为输入的原始屏幕截图
/// - `deploy_card`: 所有部署卡片信息
/// - `annotated_screen`: 标注了部署卡片位置的屏幕截图
pub struct DeployAnalyzerOutput {
    pub screen: Box<DynamicImage>,
    pub deploy_cards: Vec<DeployCard>,
    pub annotated_screen: Box<DynamicImage>,
}

pub struct DeployAnalyzer {
    use_cache: bool,
    oper_names: Vec<String>,
    matcher: BestMatcher,
    multi_match_analyzer: MultiMatchAnalyzer,
}

impl DeployAnalyzer {
    pub fn new<S: AsRef<str>, P: AsRef<Path>>(res_dir: P, opers: Vec<S>) -> Self {
        let opers_avatars = get_opers_avatars(opers, &res_dir).unwrap();
        let oper_names = opers_avatars
            .iter()
            .map(|(s, _)| s.to_string())
            .collect::<Vec<String>>();
        let images = opers_avatars
            .into_iter()
            .map(|(_, img)| img)
            .collect::<Vec<DynamicImage>>();

        let multi_match_analyzer = MultiMatchAnalyzer::new(&res_dir, "battle_deploy-card-cost1.png", None, Some(40.0))
                .roi((0.0, 0.75), (1.0, 1.0));
        Self {
            use_cache: false,
            oper_names,
            matcher: BestMatcher::new(images),
            multi_match_analyzer,
        }
    }

    pub fn use_cache(mut self) -> Self {
        self.use_cache = true;
        self
    }

    pub fn analyze_image(&mut self, image: &DynamicImage) -> Result<DeployAnalyzerOutput, String> {
        let log_tag = cformat!("<strong>[DeployAnalyzer]: </strong>");
        cprintln!("{log_tag}analyzing deploy...");
        let t = Instant::now();

        let output = self.multi_match_analyzer.analyze_image(image)?;
        let res = output.res;
        let deploy_cards: Vec<DeployCard> = res
            .rects
            .into_iter()
            .map(|rect| {
                let cropped = image.crop_imm(rect.x, rect.y, rect.width, rect.height);
                let avg_hsv_v = average_hsv_v(&cropped);
                // println!("{avg_hsv_v}");
                let available = avg_hsv_v > 90;

                let rect = Rect {
                    x: rect.x.saturating_add_signed(-15 - 40),
                    y: rect.y.saturating_add(60),
                    width: 80,
                    height: 100,
                };

                let avatar_template = image.crop_imm(rect.x, rect.y, rect.width, rect.height);
                assert!(avatar_template.width() * avatar_template.height() > 0); // make sure the template is not empty
                let res = self.matcher.match_with(avatar_template);
                let oper_name = self.oper_names.get(res).unwrap().to_string();

                DeployCard {
                    oper_name,
                    rect,
                    available,
                }
            })
            .collect();

        let annotation_t = Instant::now();
        let mut annotated_screen = output.annotated_screen;
        for deploy_card in &deploy_cards {
            let color = if deploy_card.available {
                [0, 255, 0, 255]
            } else {
                [255, 0, 0, 255]
            };
            let rect = deploy_card.rect.clone();

            draw_box(
                &mut annotated_screen,
                rect.x as i32,
                rect.y as i32,
                rect.width,
                rect.height,
                color,
            );
        }
        cprintln!("{log_tag}annotation cost: {:?}...", annotation_t.elapsed());

        cprintln!("{log_tag}total cost: {:?}...", t.elapsed());
        Ok(DeployAnalyzerOutput {
            screen: output.screen,
            deploy_cards,
            annotated_screen,
        })
    }
}

impl Analyzer for DeployAnalyzer {
    type Output = DeployAnalyzerOutput;
    fn analyze(&mut self, core: &AAH) -> Result<Self::Output, String> {
        let screen = core.screen_cap_and_cache()?;
        self.analyze_image(&screen)
    }
}

pub const EXAMPLE_DEPLOY_OPERS: [&str; 9] = [
    "char_285_medic2",
    "char_502_nblade",
    "char_500_noirc",
    "char_503_rang",
    "char_501_durin",
    "char_284_spot",
    "char_212_ansel",
    "char_208_melan",
    "char_151_myrtle",
];

#[cfg(test)]
mod test {
    use super::*;
    use crate::{vision::analyzer::Analyzer, AAH};

    #[test]
    fn test_deploy_analyzer() {
        let mut core = AAH::connect("127.0.0.1:16384", "../../resources", |_| {}).unwrap();
        let mut analyzer = DeployAnalyzer::new(&core.res_dir, EXAMPLE_DEPLOY_OPERS.to_vec());
        let output = analyzer.analyze(&mut core).unwrap();
        output.annotated_screen.save("./assets/output.png").unwrap();
        println!("{:?}", output.deploy_cards);
    }
}
