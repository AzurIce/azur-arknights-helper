use crate::{
    android, arknights,
    task::{Action, Task, TaskStep},
};
use aah_resource::level::Level;

pub struct UseSkill;
impl UseSkill {
    pub fn new(level: &Level, tile_pos: &(u32, u32)) -> Task<arknights::ActionSet> {
        let tile_pos = level.calc_tile_screen_pos(tile_pos.0, tile_pos.1, false);
        let skill_pos = level.get_skill_screen_pos();
        let task = Task::from_steps(vec![
            TaskStep::from_action(Action::detailed(android::ActionSet::click(
                tile_pos.0 as u32,
                tile_pos.1 as u32,
            ))),
            TaskStep::from_action(Action::detailed(android::ActionSet::click(
                skill_pos.0 as u32,
                skill_pos.1 as u32,
            )))
            .with_delay(0.2),
        ])
        .with_name("UseSkill");
        task
    }
}
