use image::{DynamicImage, GenericImageView};
use ocrs::ImageSource;
use regex::Regex;

use crate::{
    arknights::AahCore,
    vision::{
        analyzer::{matching::MatchOptions, multi_match::MultiMatchAnalyzer, Analyzer},
        utils::{draw_box, Rect},
    },
    CachedScreenCapper,
};

pub struct LevelAnalyzerOutput {
    pub levels: Vec<(String, Rect)>,
    pub annotated_screen: Box<DynamicImage>,
}

pub struct LevelAnalyzer {}

impl LevelAnalyzer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Analyzer<AahCore> for LevelAnalyzer {
    type Res = LevelAnalyzerOutput;
    fn analyze(&mut self, aah: &AahCore) -> anyhow::Result<Self::Res> {
        let _ = aah.screen_cap_and_cache()?;

        println!("Multimatching levels_crystal");
        // let t = Instant::now();
        let mut multi_match_analyzer =
            MultiMatchAnalyzer::new(&aah.resource.root, "levels_crystal.png").with_options(
                MatchOptions::default()
                    .with_color_mask(0..=0, 120..=200, 0..=255)
                    .with_threshold(0.94),
            );
        let res = multi_match_analyzer.analyze(aah)?;
        // println!("matched, cost {:?}", t.elapsed()); // 1s
        // res.annotated_screen.save("./test.png").unwrap();

        let mut levels = vec![];
        let mut annotated_screen = res.annotated_screen;
        for level_crystal_rect in res.res.rects {
            let x = level_crystal_rect.x + level_crystal_rect.width;
            let y = level_crystal_rect.y;
            let width = 170;
            let height = level_crystal_rect.height;

            draw_box(
                &mut annotated_screen,
                x as i32,
                y as i32,
                width,
                height,
                [0xff, 0x00, 0x00, 0x00],
            );
            let level_code_img = res.screen.crop_imm(x, y, width, height);
            let engine = &aah.ocr_engine;
            let image_source =
                ImageSource::from_bytes(level_code_img.as_bytes(), level_code_img.dimensions())
                    .unwrap();
            let ocr_input = engine.prepare_input(image_source).unwrap();

            let word_rects = engine.detect_words(&ocr_input).unwrap();
            let rects = engine.find_text_lines(&ocr_input, &word_rects);
            let texts = engine.recognize_text(&ocr_input, &rects).unwrap();
            let texts = texts
                .iter()
                .zip(rects.iter())
                .filter_map(|(text, rect)| match text {
                    Some(text) => {
                        let text = text.to_string();
                        if let Some(cap) = Regex::new(r#"[a-zA-Z\d]+-[a-zA-Z\d]+(?:-[a-zA-Z\d])*"#)
                            .unwrap()
                            .captures(&text)
                        {
                            let text = cap.get(0).unwrap().as_str().to_string();
                            Some((text, rect))
                        } else {
                            None
                        }
                    }
                    None => None,
                })
                .collect::<Vec<(String, _)>>();

            // for (text, rect) in &texts {
            //     println!("{} {:?}", text, rect)
            // }

            if let Some((text, _rect)) = texts.first() {
                let level_code_rect = Rect {
                    x,
                    y,
                    width,
                    height,
                };
                levels.push((text.to_owned(), level_code_rect.clone()));
            }
        }

        let output = LevelAnalyzerOutput {
            levels,
            annotated_screen,
        };
        Ok(output)
    }
}

#[cfg(test)]
mod test {
    use std::time::Instant;

    use super::LevelAnalyzer;

    #[test]
    fn test_level_analyzer() {
        // let aah = aah_for_test();
        // let mut analyzer = LevelAnalyzer::new();
        // println!("Analyzing...");
        // let t = Instant::now();
        // let res = analyzer.analyze(&aah).unwrap();
        // println!("Analyzed, cost {:?}", t.elapsed()); // 2.4s
        // res.annotated_screen.save("test.png").unwrap();
        // println!("{:?}", res.levels);
    }
}
