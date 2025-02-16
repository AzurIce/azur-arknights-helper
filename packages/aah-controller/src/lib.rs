//! aah-controller contains the basic device manuplating functions like
//! adb connecting, touch, swipe, adb command executing, etc.

use std::time::Duration;

use image::DynamicImage;

use crate::adb::MyError;

pub mod aah_controller;
pub mod adb_controller;
pub mod pc_controller;
pub mod adb;
pub mod app;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// 默认宽高
pub const DEFAULT_WIDTH: u32 = 1920;
pub const DEFAULT_HEIGHT: u32 = 1080;

/// [`Controller`] 承担着设备操作相关的事情，如触控、截图
/// 所有 [`Controller`]：
/// - [`AdbInputController`] 基于 adb 命令进行触控与截图
pub trait Controller {
    fn screen_size(&self) -> (u32, u32);
    /// A scale factor from the device's resolution to 1920x1080
    /// $device_res * scale_factor = 1920x1080$
    fn scale_factor(&self) -> f32 {
        self.screen_size().0 as f32 / DEFAULT_HEIGHT as f32
    }

    fn click_in_rect(&self, rect: Rect) -> Result<(), MyError> {
        let x = rand::random::<u32>() % rect.width + rect.x;
        let y = rand::random::<u32>() % rect.height + rect.y;
        self.click(x, y)
    }

    /// A scaled version of [`Controller::click_in_rect`].
    ///
    /// This scaled the coord from 1920x1080 to the actual size by simply dividing [`Controller::scale_factor`]
    fn click_in_rect_scaled(&self, rect_scaled: Rect) -> Result<(), MyError> {
        let scale_fector = self.scale_factor();
        let rect = Rect {
            x: (rect_scaled.x as f32 / scale_fector) as u32,
            y: (rect_scaled.y as f32 / scale_fector) as u32,
            width: (rect_scaled.width as f32 / scale_fector) as u32,
            height: (rect_scaled.height as f32 / scale_fector) as u32,
        };
        self.click_in_rect(rect)
    }

    fn click(&self, x: u32, y: u32) -> Result<(), MyError>;

    /// A scaled version of [`Controller::click`].
    ///
    /// This scaled the coord from 1920x1080 to the actual size by simply dividing [`Controller::scale_factor`]
    fn click_scaled(&self, x_scaled: u32, y_scaled: u32) -> Result<(), MyError> {
        let scale_factor = self.scale_factor();
        let (x, y) = (
            x_scaled as f32 / scale_factor,
            y_scaled as f32 / scale_factor,
        );
        self.click(x as u32, y as u32)
    }

    fn swipe(
        &self,
        start: (u32, u32),
        end: (i32, i32),
        duration: Duration,
        slope_in: f32,
        slope_out: f32,
    ) -> Result<(), MyError>;

    /// A scaled version of [`Controller::swipe`].
    ///
    /// This scaled the coord from 1920x1080 to the actual size by simply dividing [`Controller::scale_factor`]
    fn swipe_scaled(
        &self,
        start_scaled: (u32, u32),
        end_scaled: (i32, i32),
        duration: Duration,
        slope_in: f32,
        slope_out: f32,
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
            slope_in,
            slope_out,
        )
    }

    /// Get the raw screencap data in bytes
    fn raw_screencap(&self) -> Result<Vec<u8>, MyError>;

    /// Get the decoded screencap image
    fn screencap(&self) -> Result<image::DynamicImage, MyError>;

    /// A scaled version of [`Controller::swipe`].
    ///
    /// This scaled the screenshot image to [`DEFAULT_HEIGHT`]
    fn screencap_scaled(&self) -> Result<image::DynamicImage, MyError> {
        let screen = self.screencap()?;
        let screen = if screen.height() != DEFAULT_HEIGHT {
            // let scale_factor = 2560.0 / image.width() as f32;
            let scale_factor = DEFAULT_HEIGHT as f32 / screen.height() as f32;

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

/// A toucher contains [`Toucher::click`] and [`Toucher::swipe`]
pub trait Toucher {
    fn click_in_rect(&mut self, rect: Rect) -> anyhow::Result<()> {
        let x = rand::random::<u32>() % rect.width + rect.x;
        let y = rand::random::<u32>() % rect.height + rect.y;
        self.click(x, y)
    }

    fn click(&mut self, x: u32, y: u32) -> anyhow::Result<()>;
    fn click_scaled(
        &mut self,
        x_scaled: u32,
        y_scaled: u32,
        scale_factor: f32,
    ) -> anyhow::Result<()> {
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
    ) -> anyhow::Result<()>;

    fn swipe_scaled(
        &mut self,
        start_scaled: (u32, u32),
        end_scaled: (i32, i32),
        duration: Duration,
        slope_in: f32,
        slope_out: f32,
        scale_factor: f32,
    ) -> anyhow::Result<()> {
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

pub trait PcController {

}