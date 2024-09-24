use std::{thread, time::Duration};

use aah_cv::template_matching::MatchTemplateMethod;
use aah_resource::manifest::MatchTask;
use anyhow::Context;

use crate::vision::{
    analyzer::{levels::LevelAnalyzer, single_match::SingleMatchAnalyzer, Analyzer},
    utils::Rect,
};

use super::{
    action::{Click, ClickMatchTemplate},
    Runnable,
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

fn match_terminal_resource(aah: &crate::AAH) -> Result<Rect, anyhow::Error> {
    let mut analyzer = SingleMatchAnalyzer::new(&aah.resource.root, "terminal-resource.png")
        .roi((0.0, 0.875), (1.0, 1.0));
    let res = analyzer
        .analyze(aah)
        .map_err(|err| anyhow::anyhow!(err))
        .context("match terminal-resource")?;
    res.res
        .rect
        .ok_or(anyhow::anyhow!("failed to match terminal-resource"))
}

fn match_levels_resources_lmb(aah: &crate::AAH) -> Result<Rect, anyhow::Error> {
    let mut analyzer = SingleMatchAnalyzer::new(&aah.resource.root, "levels-resources-lmb.png")
        .roi((0.0, 0.5), (1.0, 0.75));
    let res = analyzer
        .analyze(aah)
        .map_err(|err| anyhow::anyhow!(err))
        .context("match levels-resources-lmb")?;
    res.res.matched_img.save("matched.png").unwrap();
    res.annotated_screen.save("test.png").unwrap();
    res.res
        .rect
        .ok_or(anyhow::anyhow!("failed to match levels-resources-lmb"))
}

fn analyze_levels(aah: &crate::AAH) -> Result<Vec<(String, Rect)>, anyhow::Error> {
    let mut analyzer = LevelAnalyzer::new();
    let res = analyzer.analyze(aah).map_err(|err| anyhow::anyhow!(err))?;
    println!("{:?}", res.levels);
    Ok(res.levels)
}

impl Runnable for ChooseLevel {
    type Err = anyhow::Error;
    fn run(&self, aah: &crate::AAH) -> Result<Self::Res, Self::Err> {
        aah.emit_task_evt(super::TaskEvt::Log("entering terminal page".to_string()));
        ClickMatchTemplate::new(MatchTask::Template("main_terminal.png".to_string()))
            .run(aah)
            .map_err(|err| anyhow::anyhow!(err))?;

        thread::sleep(Duration::from_millis(800));
        if self.0.starts_with("CE") {
            aah.emit_task_evt(super::TaskEvt::Log(
                "entering terminal-resource page".to_string(),
            ));
            let rect = match_terminal_resource(aah)?;
            aah.click_in_rect(rect)?;
            thread::sleep(Duration::from_millis(800));

            aah.emit_task_evt(super::TaskEvt::Log("entering levels-lmb page".to_string()));
            let rect = match_levels_resources_lmb(aah)?;
            aah.click_in_rect(rect)?;
            thread::sleep(Duration::from_millis(800));

            let levels = analyze_levels(aah)?;
            if let Some((_, rect)) = levels.iter().find(|(level, _)| level == &self.0) {
                Click::new(rect.x + rect.width / 2, rect.y + rect.height / 2)
                    .run(aah)
                    .map_err(|err| anyhow::anyhow!(err))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use aah_resource::LocalResource;

    use crate::{task::{choose_level::analyze_levels, Runnable}, AAH};

    use super::{match_levels_resources_lmb, ChooseLevel};

    #[test]
    fn test_analyze_levels() -> Result<(), anyhow::Error> {
        let resource = LocalResource::load("../../resources").unwrap();
        let aah = AAH::connect("127.0.0.1:16384", Arc::new(resource.into())).unwrap();
        let res = analyze_levels(&aah);
        println!("{:?}", res);
        Ok(())
    }

    #[test]
    fn test_match_levels_resources_lmb() -> Result<(), anyhow::Error> {
        let resource = LocalResource::load("../../resources").unwrap();
        let aah = AAH::connect("127.0.0.1:16384", Arc::new(resource.into())).unwrap();
        let res = match_levels_resources_lmb(&aah);
        println!("{:?}", res);
        Ok(())
    }

    #[test]
    fn test_choose_level() {
        let resource = LocalResource::load("../../resources").unwrap();
        let aah = AAH::connect("127.0.0.1:16384", Arc::new(resource.into())).unwrap();
        let task = ChooseLevel::new("CE-5");
        task.run(&aah).unwrap();
    }
}
