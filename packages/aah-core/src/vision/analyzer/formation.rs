use crate::{task::{match_task::MatchTask, Task}, AAH};

use super::Analyzer;

#[derive(Debug)]
pub struct SquadAnalyzerOutput {

}

pub struct SquadAnalyzer {

}

impl Analyzer for SquadAnalyzer {
    type Output = SquadAnalyzerOutput;
    fn analyze(&mut self, core: &AAH) -> Result<Self::Output, String> {
        // Make sure that we are in the operation-start page
        MatchTask::Template("operation-start_start.png".to_string()).run(core)?;
        
    }
}