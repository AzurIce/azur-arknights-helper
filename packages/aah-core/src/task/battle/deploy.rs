use std::time::Duration;

use aah_resource::level::Level;

use crate::{
    task::{action::Swipe, copilot::Direction, Task, TaskStep},
    vision::utils::Rect,
};

pub struct Deploy;

impl Deploy {
    pub fn new(
        level: &Level,
        deploy_card_rect: &Rect,
        tile_pos: &(u32, u32),
        direction: &Direction,
    ) -> Task {
        let tile_pos = level.calc_tile_screen_pos(tile_pos.0, tile_pos.1, true);
        let tile_pos = (tile_pos.0 as u32, tile_pos.1 as u32);
        let swipe_delta = 400;
        let swipe_end = match direction {
            Direction::Up => (tile_pos.0 as i32, tile_pos.1 as i32 - swipe_delta),
            Direction::Right => (tile_pos.0 as i32 + swipe_delta, tile_pos.1 as i32),
            Direction::Down => (tile_pos.0 as i32, tile_pos.1 as i32 + swipe_delta),
            Direction::Left => (tile_pos.0 as i32 - swipe_delta, tile_pos.1 as i32),
        };
        let task = Task::from_steps(vec![
            TaskStep::from_action(Swipe::new(
                (deploy_card_rect.x, deploy_card_rect.y),
                (tile_pos.0 as i32, tile_pos.1 as i32),
                Duration::from_secs_f32(0.2),
                0.0,
                0.0,
            )),
            TaskStep::from_action(Swipe::new(
                tile_pos,
                swipe_end,
                Duration::from_secs_f32(0.2),
                0.0,
                0.0,
            ))
            .with_delay(0.2),
        ])
        .with_name("Deploy");
        task
    }
}
