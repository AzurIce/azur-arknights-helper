use crate::task::{action::Click, Task, TaskStep};
use aah_resource::level::Level;

pub struct Retreat;

impl Retreat {
    pub fn new(level: &Level, tile_pos: &(u32, u32)) -> Task {
        let tile_pos = level.calc_tile_screen_pos(tile_pos.0, tile_pos.1, false);
        let retreat_pos = level.get_retreat_screen_pos();
        let task = Task::from_steps(vec![
            TaskStep::action(Click::new(tile_pos.0 as u32, tile_pos.1 as u32)),
            TaskStep::action(Click::new(retreat_pos.0 as u32, retreat_pos.1 as u32))
                .deplay_sec_f32(0.2),
        ])
        .with_name("Retreat");
        task
    }
}
