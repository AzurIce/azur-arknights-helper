use aah_resource::level::Level;

use crate::{config::task::{Task, TaskStep}, task::action::Action};

pub struct Retreat;

impl Retreat {
    pub fn new(level: &Level, tile_pos: &(u32, u32)) -> Task {
        let tile_pos = level.calc_tile_screen_pos(tile_pos.0, tile_pos.1, false);
        let retreat_pos = level.get_retreat_screen_pos();
        let task = Task::from_steps(vec![
            TaskStep::action(Action::ActionClick {
                x: tile_pos.0 as u32,
                y: tile_pos.1 as u32,
            }),
            TaskStep::action(Action::ActionClick {
                x: retreat_pos.0 as u32,
                y: retreat_pos.1 as u32,
            })
            .deplay_sec_f32(0.2),
        ])
        .with_name("Retreat");
        task
    }
}
