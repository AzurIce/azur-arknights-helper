use crate::AAH;

use super::{multi_template_match::MultiTemplateMatchAnalyzer, Analyzer};

#[derive(Debug)]
pub struct DeployAnalyzerOutput {}

pub struct DeployAnalyzer {}

impl Analyzer for DeployAnalyzer {
    type Output = DeployAnalyzerOutput;
    fn analyze(&mut self, core: &AAH) -> Result<Self::Output, String> {
        // Make sure that we are in the operation-start page
        let cost_pos_analyzer =
            MultiTemplateMatchAnalyzer::new("battle_deploy-card-cost".to_string())
                .analyze(core)
                .unwrap();
        println!("{:?}", cost_pos_analyzer);
        Ok(DeployAnalyzerOutput {  })
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
