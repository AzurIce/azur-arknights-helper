use std::time::Duration;

use image::math::Rect;

use crate::adb::MyError;

pub mod adb_input_controller;
pub use adb_input_controller::AdbInputController;
pub mod minitouch_controller;
pub use minitouch_controller::MiniTouchController;

pub trait Controller {
    fn click_in_rect(&self, rect: Rect) -> Result<(), MyError> {
        let x = rand::random::<u32>() % rect.width + rect.x;
        let y = rand::random::<u32>() % rect.height + rect.y;
        self.click(x, y)
    }

    fn click(&self, x: u32, y: u32) -> Result<(), MyError>;

    fn swipe(&self, start: (u32, u32), end: (i32, i32), duration: Duration) -> Result<(), MyError>;

    fn screencap(&self) -> Result<image::DynamicImage, MyError>;

    fn press_home(&self) -> Result<(), MyError>;

    fn press_esc(&self) -> Result<(), MyError>;
}

pub trait Toucher {
    fn click_in_rect(&mut self, rect: Rect) -> Result<(), String> {
        let x = rand::random::<u32>() % rect.width + rect.x;
        let y = rand::random::<u32>() % rect.height + rect.y;
        self.click(x, y)
    }

    fn click(&mut self, x: u32, y: u32) -> Result<(), String>;

    fn swipe(&mut self, start: (u32, u32), end: (i32, i32), duration: Duration, slope_in: bool, slope_out: bool) -> Result<(), String>;
}