use std::{thread, time::Duration};

use anyhow::Context;
use auto_play::{
    actions::{Click, ClickMatchTemplate, Runnable},
    HasController,
};
use image::math::Rect;
use image::DynamicImage;

use crate::{
    analyzer::levels::LevelAnalyzer,
    vision::analyzer::{
        matching::MatchOptions,
        single_match::{SingleMatchAnalyzer, SingleMatchAnalyzerOutput},
    },
    AahCore,
};

/// A task to choose level from main
///
/// the inner String is the level code
pub struct ChooseLevel(String);

impl ChooseLevel {
    pub fn new(level_code: impl AsRef<str>) -> Self {
        Self(level_code.as_ref().to_string())
    }
}

fn match_terminal_resource(aah: &AahCore) -> Result<Rect, anyhow::Error> {
    let analyzer = SingleMatchAnalyzer::new(&aah.resource.root, "terminal-resource.png")
        .with_options(MatchOptions::default().with_roi((0.0, 0.875), (1.0, 1.0)));
    let SingleMatchAnalyzerOutput { res, .. } = analyzer.execute(aah)?;
    res.result
        .map(|m| m.rect)
        .ok_or(anyhow::anyhow!("failed to match terminal-resource"))
}

fn match_levels_resources_lmb(aah: &AahCore) -> Result<Rect, anyhow::Error> {
    let analyzer = SingleMatchAnalyzer::new(&aah.resource.root, "levels-resources-lmb.png")
        .with_options(MatchOptions::default().with_roi((0.0, 0.5), (1.0, 0.75)));
    let SingleMatchAnalyzerOutput {
        res,
        annotated_screen,
        ..
    } = analyzer
        .execute(aah)
        .map_err(|err| anyhow::anyhow!(err))
        .context("match levels-resources-lmb")?;
    DynamicImage::from(res.matched_image)
        .save("matched.png")
        .unwrap();
    annotated_screen.save("test.png").unwrap();
    res.result
        .map(|m| m.rect)
        .ok_or(anyhow::anyhow!("failed to match levels-resources-lmb"))
}

fn analyze_levels(aah: &AahCore) -> Result<Vec<(String, Rect)>, anyhow::Error> {
    let analyzer = LevelAnalyzer::new();
    let res = analyzer.execute(aah)?;
    println!("{:?}", res.levels);
    Ok(res.levels)
}

impl Runnable<AahCore> for ChooseLevel {
    type Output = ();
    fn execute(&self, executor: &AahCore) -> anyhow::Result<Self::Output> {
        // aah.emit_task_evt(super::TaskEvt::Log("entering terminal page".to_string()));
        ClickMatchTemplate::new("main_terminal.png")
            .execute(executor)
            .map_err(|err| anyhow::anyhow!(err))?;

        thread::sleep(Duration::from_millis(800));
        if self.0.starts_with("CE") {
            // aah.emit_task_evt(super::TaskEvt::Log(
            //     "entering terminal-resource page".to_string(),
            // ));
            let rect = match_terminal_resource(executor)?.into();
            executor.controller().click_in_rect(rect)?;
            thread::sleep(Duration::from_millis(800));

            // aah.emit_task_evt(super::TaskEvt::Log("entering levels-lmb page".to_string()));
            let rect = match_levels_resources_lmb(executor)?.into();
            executor.controller().click_in_rect(rect)?;
            thread::sleep(Duration::from_millis(800));

            let levels = analyze_levels(executor)?;
            if let Some((_, rect)) = levels.iter().find(|(level, _)| level == &self.0) {
                Click::new(rect.x + rect.width / 2, rect.y + rect.height / 2)
                    .execute(executor)
                    .map_err(|err| anyhow::anyhow!(err))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_analyze_levels() -> Result<(), anyhow::Error> {
        // let resource = LocalResource::load("../../resources").unwrap();
        // let aah = AAH::connect("127.0.0.1:16384", Arc::new(resource.into())).unwrap();
        // let res = analyze_levels(&aah);
        // println!("{:?}", res);
        Ok(())
    }

    #[test]
    fn test_match_levels_resources_lmb() -> Result<(), anyhow::Error> {
        // let resource = LocalResource::load("../../resources").unwrap();
        // let aah = AAH::connect("127.0.0.1:16384", Arc::new(resource.into())).unwrap();
        // let res = match_levels_resources_lmb(&aah);
        // println!("{:?}", res);
        Ok(())
    }

    #[test]
    fn test_choose_level() {
        // let resource = LocalResource::load("../../resources").unwrap();
        // let aah = AAH::connect("127.0.0.1:16384", Arc::new(resource.into())).unwrap();
        // let task = ChooseLevel::new("CE-5");
        // task.run(&aah).unwrap();
    }
}
