use aah_resource::level::Level;

use crate::{config::task::{Task, TaskStep}, task::action::Action};

pub struct UseSkill;
impl UseSkill {
    pub fn new(level: &Level, tile_pos: &(u32, u32)) -> Task {
        let tile_pos = level.calc_tile_screen_pos(tile_pos.0, tile_pos.1, false);
        let skill_pos = level.get_skill_screen_pos();
        let task = crate::config::task::Task::from_steps(vec![
            TaskStep::action(Action::ActionClick {
                x: tile_pos.0 as u32,
                y: tile_pos.1 as u32,
            }),
            TaskStep::action(Action::ActionClick {
                x: skill_pos.0 as u32,
                y: skill_pos.1 as u32,
            })
            .deplay_sec_f32(0.2),
        ])
        .with_name("UseSkill");
        task
    }
}
