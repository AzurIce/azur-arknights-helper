use std::time::Duration;

use image::{math::Rect, DynamicImage};
use nalgebra::coordinates::X;

use crate::adb::MyError;

pub mod adb_input_controller;
pub use adb_input_controller::AdbInputController;
pub mod minitouch_controller;
pub use minitouch_controller::MiniTouchController;

pub trait Controller {
    fn scale_factor(&self) -> f32;

    fn click_in_rect(&self, rect: Rect) -> Result<(), MyError> {
        let x = rand::random::<u32>() % rect.width + rect.x;
        let y = rand::random::<u32>() % rect.height + rect.y;
        self.click(x, y)
    }

    fn click_in_rect_scaled(&self, rect_scaled: Rect) -> Result<(), MyError> {
        let scale_fector = self.scale_factor();
        let rect = Rect {
            x: (rect_scaled.x as f32 / scale_fector) as u32,
            y: (rect_scaled.y as f32 / scale_fector) as u32,
            width: (rect_scaled.width as f32 / scale_fector) as u32,
            height: (rect_scaled.height as f32 / scale_fector) as u32
        };
        self.click_in_rect(rect)
    }

    fn click(&self, x: u32, y: u32) -> Result<(), MyError>;

    fn click_scaled(&self, x_scaled: u32, y_scaled: u32) -> Result<(), MyError> {
        let scale_factor = self.scale_factor();
        let (x, y) = (
            x_scaled as f32 / scale_factor,
            y_scaled as f32 / scale_factor,
        );
        self.click(x as u32, y as u32)
    }

    fn swipe(&self, start: (u32, u32), end: (i32, i32), duration: Duration) -> Result<(), MyError>;

    fn swipe_scaled(
        &self,
        start_scaled: (u32, u32),
        end_scaled: (i32, i32),
        duration: Duration,
    ) -> Result<(), MyError> {
        let scale_factor = self.scale_factor();
        let (start, end) = (
            (
                start_scaled.0 as f32 / scale_factor,
                start_scaled.1 as f32 / scale_factor,
            ),
            (
                end_scaled.0 as f32 / scale_factor,
                end_scaled.1 as f32 / scale_factor,
            ),
        );
        self.swipe(
            (start.0 as u32, start.1 as u32),
            (end.0 as i32, end.1 as i32),
            duration,
        )
    }

    fn screencap(&self) -> Result<image::DynamicImage, MyError>;

    fn screencap_scaled(&self) -> Result<image::DynamicImage, MyError> {
        let screen = self.screencap()?;
        let screen = if screen.height() != 1440 {
            // let scale_factor = 2560.0 / image.width() as f32;
            let scale_factor = 1440.0 / screen.height() as f32;

            let new_width = (screen.width() as f32 * scale_factor) as u32;
            let new_height = (screen.height() as f32 * scale_factor) as u32;

            DynamicImage::from(image::imageops::resize(
                &screen,
                new_width,
                new_height,
                image::imageops::FilterType::Triangle,
            ))
        } else {
            screen
        };
        Ok(screen)
    }

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
    fn click_scaled(
        &mut self,
        x_scaled: u32,
        y_scaled: u32,
        scale_factor: f32,
    ) -> Result<(), String> {
        let (x, y) = (
            x_scaled as f32 * scale_factor,
            y_scaled as f32 * scale_factor,
        );
        self.click(x as u32, y as u32)
    }

    fn swipe(
        &mut self,
        start: (u32, u32),
        end: (i32, i32),
        duration: Duration,
        slope_in: f32,
        slope_out: f32,
    ) -> Result<(), String>;

    fn swipe_scaled(
        &mut self,
        start_scaled: (u32, u32),
        end_scaled: (i32, i32),
        duration: Duration,
        slope_in: f32,
        slope_out: f32,
        scale_factor: f32,
    ) -> Result<(), String> {
        let (start, end) = (
            (
                start_scaled.0 as f32 * scale_factor,
                start_scaled.1 as f32 * scale_factor,
            ),
            (
                end_scaled.0 as f32 * scale_factor,
                end_scaled.1 as f32 * scale_factor,
            ),
        );
        self.swipe(
            (start.0 as u32, start.1 as u32),
            (end.0 as i32, end.1 as i32),
            duration,
            slope_in,
            slope_out,
        )
    }
}
