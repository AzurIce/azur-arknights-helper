use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use color_print::{cformat, cprintln};
use image::DynamicImage;
use serde::Serialize;

use crate::{
    arknights::AahCore,
    utils::resource::get_opers_avatars,
    vision::{
        analyzer::{matching::MatchOptions, multi_match::MultiMatchAnalyzer, Analyzer},
        matcher::best_matcher::BestMatcher,
        utils::{average_hsv_v, draw_box, Rect},
    },
    CachedScreenCapper,
};

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
    res_dir: PathBuf,

    use_cache: bool,
    oper_names: Vec<String>,
    matcher: BestMatcher,
    multi_match_analyzer: MultiMatchAnalyzer,
}

impl DeployAnalyzer {
    pub fn new<S: AsRef<str>>(res_dir: impl AsRef<Path>, opers: &[S]) -> Self {
        let res_dir = res_dir.as_ref().to_path_buf();

        let opers_avatars = get_opers_avatars(opers, &res_dir).unwrap();
        let oper_names = opers_avatars
            .iter()
            .map(|(s, _)| s.to_string())
            .collect::<Vec<String>>();
        let images = opers_avatars
            .into_iter()
            .map(|(_, img)| img)
            .collect::<Vec<DynamicImage>>();

        // ccorr_normed 0.9
        let multi_match_analyzer =
            MultiMatchAnalyzer::new(&res_dir, "battle_deploy-card-cost1.png")
                .with_options(MatchOptions::default().with_roi((0.0, 0.75), (1.0, 1.0)));
        Self {
            res_dir,
            use_cache: false,
            oper_names,
            matcher: BestMatcher::new(images, None),
            multi_match_analyzer,
        }
    }

    pub fn use_cache(mut self) -> Self {
        self.use_cache = true;
        self
    }

    pub fn analyze_image(&self, image: &DynamicImage) -> anyhow::Result<DeployAnalyzerOutput> {
        let log_tag = cformat!("<strong>[DeployAnalyzer]: </strong>");
        cprintln!("{log_tag}analyzing deploy...");
        let t = Instant::now();

        cprintln!("{log_tag}searching deploy cards...");
        let output = self.multi_match_analyzer.analyze_image(image)?;
        let res = output.res;
        cprintln!("{log_tag}found {} deploy cards...", res.rects.len());
        let mut deploy_cards: Vec<DeployCard> = Vec::with_capacity(res.rects.len());
        for rect in res.rects {
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
            if let Some(idx) = res {
                let oper_name = self.oper_names.get(idx).unwrap().to_string();
                deploy_cards.push(DeployCard {
                    oper_name,
                    rect,
                    available,
                })
            }
        }
        cprintln!("{log_tag}deploy_cards elapsed: {:?}...", t.elapsed());
        cprintln!("{log_tag}deploy_cards:{:?}", deploy_cards);

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
        cprintln!("{log_tag}annoteded elapsed: {:?}...", t.elapsed());

        cprintln!("{log_tag}total cost: {:?}...", t.elapsed());
        Ok(DeployAnalyzerOutput {
            screen: output.screen,
            deploy_cards,
            annotated_screen,
        })
    }
}

impl Analyzer<AahCore> for DeployAnalyzer {
    type Res = DeployAnalyzerOutput;
    fn analyze(&mut self, core: &AahCore) -> anyhow::Result<Self::Res> {
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

    #[test]
    fn print_oper_list() {}

    #[test]
    fn test_deploy_analyzer() {
        // let mut core = AAH::connect("127.0.0.1:16384", "../../resources", |_| {}).unwrap();
        let mut analyzer = DeployAnalyzer::new(
            "../../resources",
            &[
                // "char_1028_texas2",
                // "char_4087_ines",
                // "char_479_sleach",
                // "char_222_bpipe",
                // "char_1016_agoat2",
                // "char_245_cello",
                // "char_1020_reed2",
                // "char_4117_ray",
                // "char_2025_shu",
                // "char_1032_excu2",
                // "char_1035_wisdel",
                // "char_311_mudrok",
                "char_1016_agoat2",
                "char_213_mostma",
                "char_377_gdglow",
                "char_1034_jesca2",
                "char_1035_wisdel",
                "char_350_surtr",
                "char_1029_yato2",
                "char_1020_reed2",
                "char_293_thorns",
                "char_136_hsguma",
                "char_264_f12yin",
                "char_4087_ines",
            ],
        );
        // let mut analyzer = DeployAnalyzer::new(&core.res_dir, core.default_oper_list.clone()); // self.default_oper_list.clone() cost 52s
        // let image = image::open("../../resources/templates/MUMU-1920x1080/1-4.png").unwrap();
        let image = image::open("./assets/5-10-resumed.png").unwrap();
        let output = analyzer.analyze_image(&image).unwrap();
        output
            .annotated_screen
            .save("./assets/output-5-10-resumed.png")
            .unwrap();
        println!("{:?}", output.deploy_cards);
        let image = image::open("./assets/5-10-paused.png").unwrap();
        let output = analyzer.analyze_image(&image).unwrap();
        output
            .annotated_screen
            .save("./assets/output-5-10-paused.png")
            .unwrap();
        println!("{:?}", output.deploy_cards);
    }
}
