use aah_cv::template_matching::MatchTemplateMethod;

use crate::vision::utils::Rect;

use super::{multi_match::MultiMatchAnalyzer, Analyzer};

pub struct LevelAnalyzerOutput {
    levels: Vec<(String, Rect)>,
}

pub struct LevelAnalyzer {}

impl LevelAnalyzer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Analyzer for LevelAnalyzer {
    type Output = LevelAnalyzerOutput;
    fn analyze(&mut self, aah: &crate::AAH) -> Result<Self::Output, String> {
        let screen = aah.screen_cap_and_cache()?;

        let mut multi_match_analyzer =
            MultiMatchAnalyzer::new(&aah.resource.root, "levels_crystal.png")
                .color_mask(0..=0, 120..=200, 0..=255)
                .threshold(0.94);
        let res = multi_match_analyzer.analyze(aah)?;
        res.annotated_screen.save("./test.png").unwrap();
        // println!("{:?}", res.res.rects);
        let output = LevelAnalyzerOutput { levels: vec![] };
        Ok(output)
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use aah_resource::LocalResource;

    use crate::{vision::analyzer::Analyzer, AAH};

    use super::LevelAnalyzer;

    #[test]
    fn test_level_analyzer() {
        let resource = LocalResource::load("../../resources").unwrap();
        let aah = AAH::connect("127.0.0.1:16384", Arc::new(resource.into())).unwrap();
        let mut analyzer = LevelAnalyzer::new();
        analyzer.analyze(&aah);
    }
}
