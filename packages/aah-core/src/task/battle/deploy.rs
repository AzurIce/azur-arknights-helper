use aah_resource::{level::Level, manifest::Action};

use crate::vision::utils::Rect;
use aah_resource::manifest::{
    copilot::Direction,
    task::{Task, TaskStep},
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
            TaskStep::action(Action::ActionSwipe {
                p1: (deploy_card_rect.x, deploy_card_rect.y),
                p2: (tile_pos.0 as i32, tile_pos.1 as i32),
                duration: 0.2,
                slope_in: 0.0,
                slope_out: 0.0,
            }),
            TaskStep::action(Action::ActionSwipe {
                p1: tile_pos,
                p2: swipe_end,
                duration: 0.2,
                slope_in: 0.0,
                slope_out: 0.0,
            })
            .deplay_sec_f32(0.2),
        ])
        .with_name("Deploy");
        task
    }
}
