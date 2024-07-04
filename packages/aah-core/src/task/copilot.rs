use std::{thread, time::Duration};

use aah_resource::level::get_level;
use color_print::{cformat, cprintln};
use serde::{Deserialize, Serialize};

use super::{builtins::ActionClickMatch, match_task::MatchTask, Task};

#[derive(Debug, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CopilotTask {
    level_id: String,
    actions: Vec<CopilotActions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CopilotActions {
    SpeedUp,
    Deploy {
        name: String,
        tile_pos: (u32, u32),
        direction: Direction,
    },
}

impl Task for CopilotTask {
    type Err = String;
    fn run(&self, aah: &crate::AAH) -> Result<Self::Res, Self::Err> {
        let log_tag = cformat!("<strong>[CopilotTask {}]: </strong>", self.level_id);
        let level = get_level(self.level_id.as_str(), aah.res_dir.join("levels.json")).unwrap();

        // disable prts
        cprintln!("{log_tag}disabling prts...");
        let disable_prts_task =
            ActionClickMatch::new(MatchTask::Template("prts-enabled.png".to_string()), None);
        match disable_prts_task.run(aah) {
            Ok(_) => cprintln!("disabled prts"),
            Err(err) => cprintln!("failed to disable prts: {:?}, skipping", err),
        }

        cprintln!("{log_tag}clicking start-pre...");
        let start_pre =
            ActionClickMatch::new(MatchTask::Template("level_start-pre.png".to_string()), None);
        match start_pre.run(aah) {
            Ok(_) => cprintln!("{log_tag}<g>clicked start pre</g>"),
            Err(err) => {
                let err = format!("failed to click start pre: {}", err);
                cprintln!("{log_tag}<r>{}</r>", err);
                return Err(err);
            }
        }

        thread::sleep(Duration::from_secs_f32(0.5));
        // TODO: formation

        cprintln!("{log_tag}clicking start...");
        let start_pre =
            ActionClickMatch::new(MatchTask::Template("formation_start.png".to_string()), None);
        match start_pre.run(aah) {
            Ok(_) => cprintln!("{log_tag}<g>clicked start</g>"),
            Err(err) => {
                let err = format!("failed to click start: {}", err);
                cprintln!("{log_tag}<r>{}</r>", err);
                return Err(err);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{task::Task, AAH};

    use super::{CopilotActions, CopilotTask};

    fn task() -> CopilotTask {
        CopilotTask {
            level_id: "1-4".to_string(),
            actions: vec![CopilotActions::SpeedUp],
        }
    }

    #[test]
    fn foo() {
        let aah = AAH::connect("127.0.0.1:16384", "../../resources", |_| {}).unwrap();
        let task = task();
        task.run(&aah).unwrap();
    }
}
