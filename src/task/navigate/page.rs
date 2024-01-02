use image::math::Rect;

use crate::{
    controller::Controller,
    task::{AndTask, MatchTask, OrTask, ExecResult, Exec},
};

pub enum TerminalTag {
    Terminal,
    MainTheme,
    SideStory,
    SideSideStory,
    Resource,
    // Tasks,
    Rougue,
    // ContingencyContract,
}

// pub enum NormalPage {
//     Main,
//     // Squads, // 似乎不太用
//     // Operator,
//     // Archive,
//     Terminal(TerminalTag),
//     Base,
//     Recruit,
//     // HeadHunt, // 似乎也不太用
//     // Store,
//     Mission,
//     Friend,

//     // Storage,
//     UnknownHasTopNavigator,
// }

// impl Page for NormalPage {
//     fn identifier_task(&self) -> Box<dyn Task> {
//         match self {
//             NormalPage::Main => {
//                 Box::new(MatchTask::Template("EnterInfrastMistCity.png".to_string()))
//             }
//             // NormalPage::Squads => todo!(),
//             // NormalPage::Operator => todo!(),
//             // NormalPage::Archive => todo!(),
//             NormalPage::Terminal(_) => Box::new(OrTask::new(vec![
//                 Box::new(MatchTask::Template("TerminalTagResource.png".to_string())),
//                 Box::new(MatchTask::Template(
//                     "TerminalTagResourceActive.png".to_string(),
//                 )),
//             ])),
//             NormalPage::Base => Box::new(MatchTask::Template("BaseButtonOverview.png".to_string())),
//             NormalPage::Recruit => {
//                 Box::new(MatchTask::Template("BaseButtonOverview.png".to_string()))
//             }

//             // NormalPage::HeadHunt => todo!(),
//             // NormalPage::Store => todo!(),
//             NormalPage::Mission => Box::new(OrTask::new(vec![
//                 Box::new(MatchTask::Template("MissonTagMainTheme.png".to_string())),
//                 Box::new(MatchTask::Template(
//                     "MissonTagMainThemeActive.png".to_string(),
//                 )),
//             ])),
//             NormalPage::Friend => Box::new(OrTask::new(vec![
//                 Box::new(MatchTask::Template("ButtonPersonalCardActive.png".to_string())),
//                 Box::new(MatchTask::Template("ButtonPersonalCard.png".to_string())),
//             ])),
//             // NormalPage::Storage => ,
//             NormalPage::UnknownHasTopNavigator => Box::new(MatchTask::Template(
//                 "ButtonToggleTopNavigator.png".to_string(),
//             )),
//         }
//     }
//     fn top_navigator(&self) -> bool {
//         match self {
//             NormalPage::Terminal(_)
//             | NormalPage::Base
//             | NormalPage::Recruit
//             | NormalPage::Friend
//             | NormalPage::Mission
//             | Self::UnknownHasTopNavigator => true,
//             _ => false,
//         }
//     }
// }

// pub trait Page {
//     fn identifier_task(&self) -> Box<dyn Task>;
//     fn top_navigator(&self) -> bool;
// }

pub struct Page {
    name: String,
    parents: Vec<Page>,
    childrens: Vec<Page>,
}

#[cfg(test)]
mod test {
    use std::error::Error;

    // use crate::{controller::Controller, task::navigate::page::{NormalPage, Page}};

    #[test]
    fn test_main_page_check() -> Result<(), Box<dyn Error>> {
        // let controller = Controller::connect("127.0.0.1:16384")?;
        // let res = NormalPage::Main.identifier_task().run(&controller).is_ok();
        // println!("{res}");
        Ok(())
    }
}