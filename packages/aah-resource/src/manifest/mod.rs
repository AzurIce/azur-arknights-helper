pub mod copilot;
pub mod navigate;
pub mod task;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Action {
    ByName(String),
    // Multi(Multi),
    // Action
    ActionPressEsc,
    ActionPressHome,
    ActionClick {
        x: u32,
        y: u32,
    },
    ActionSwipe {
        p1: (u32, u32),
        p2: (i32, i32),
        duration: f32,
        slope_in: f32,
        slope_out: f32,
    },
    ActionClickMatch {
        #[serde(flatten)]
        match_task: MatchTask,
    },
    // Navigate
    NavigateIn(String),
    NavigateOut(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "template")]
pub enum MatchTask {
    Template(String), // template_filename
                      // Ocr(String),      // text
}

/// 对应 `resource/version.json`
#[derive(Debug, Deserialize)]
pub struct Manifest {
    #[serde(with = "time::serde::rfc3339")]
    pub last_updated: OffsetDateTime,
}

impl PartialEq for Manifest {
    fn eq(&self, other: &Self) -> bool {
        self.last_updated == other.last_updated
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deserialize() {
        let json = r#"{
    "last_updated": "2024-09-18T22:52:42.3862364+08:00"
}"#;

        println!("{}", json);
        let version: Manifest = serde_json::from_str(json).unwrap();
        println!("{:?}", version);
    }
}