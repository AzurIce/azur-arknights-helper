use crate::{actions::copilot::Direction, Action};
use aah_resource::level::Level;
use auto_play::task::{Task, TaskStep};

pub struct Retreat;

impl Retreat {
    pub fn new(level: &Level, tile_pos: &(u32, u32)) -> Task<Action> {
        let tile_pos = level.calc_tile_screen_pos(tile_pos.0, tile_pos.1, false);
        let retreat_pos = level.get_retreat_screen_pos();
        let task = Task::from_steps(vec![
            TaskStep::from_action(Action::click(tile_pos.0 as u32, tile_pos.1 as u32)),
            TaskStep::from_action(Action::click(retreat_pos.0 as u32, retreat_pos.1 as u32))
                .with_delay(0.2),
        ])
        .with_name("Retreat");
        task
    }
}
