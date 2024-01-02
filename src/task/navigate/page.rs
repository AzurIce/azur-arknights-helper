

pub enum NormalPage {
    Main,
    Terminal,
    Squads,
    Operator,
    Store,
    Recruit,
    HeadHunt,
    Mission,
    Base,
    Storage
}

// impl ToString for NormalPage {
//     fn to_string(&self) -> String {
//         match self {

//         }
//     }
// }

trait Page {
    fn identifier_match_tasks() -> Vec<MatchTask>;
    fn check(controller: &Controller) -> bool {
        MultipleMatchTask::new(Self::identifier_match_tasks()).run(controller).is_ok()
    }
}

#[cfg(test)]
mod test {
    use std::error::Error;

    use crate::controller::Controller;

    use super::{MainPage, Page};

    #[test]
    fn test_main_page_check() -> Result<(), Box<dyn Error>> {
        let controller = Controller::connect("127.0.0.1:16384")?;
        let res = MainPage::check(&controller);
        assert!(res);
        Ok(())
    }
}

#[derive(Default)]
pub struct MainPage {
}

impl Page for MainPage {
    fn identifier_match_tasks() -> Vec<MatchTask> {
        vec![MatchTask::Template("EnterRecruitMistCity.png".to_string())]
    }
}