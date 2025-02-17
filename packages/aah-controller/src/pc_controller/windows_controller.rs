use std::{thread, time::Duration};

use crate::{adb::MyError, Controller, PcControllerTrait, WindowInfo};

use windows::Win32::{Foundation::{BOOL, HWND, LPARAM, RECT}, Graphics::Gdi::{BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY}, UI::WindowsAndMessaging::{
        EnumWindows, GetDesktopWindow, GetSystemMetrics, GetWindowRect, GetWindowTextW, IsWindowVisible, SetForegroundWindow, SM_CXSCREEN, SM_CYSCREEN
    }};
use enigo::{
    Button, Coordinate::{Abs, Rel}, Direction::{Click, Press, Release}, Enigo, Key, Keyboard, Mouse, Settings
};

pub fn create_pc_controller() -> Result<Box<dyn PcControllerTrait + Sync + Send>, MyError> {
    println!("PcController connecting in platform: windows");

    let controller = WindowsController::new();

    Ok(Box::new(controller))
}

struct WindowsController {
    width: u32,
    height: u32,
}

impl WindowsController {
    fn new() -> Self {
        println!("WindowsController created");

        if Self::true_width() != 1920 || Self::true_height() != 1080 {
            panic!("WindowsController only supports 1920x1080 screen resolution, but got {}x{}", Self::true_width(), Self::true_height());
        }

        Self {
            width: Self::true_width(),
            height: Self::true_height(),
        }
    }
}

#[allow(unused_variables)]
impl Controller for WindowsController {
    fn screen_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn click(&self, x: u32, y: u32) -> Result<(), MyError> {
        self.impl_left_click(x as i32, y as i32)
    }

    fn swipe(
        &self,
        start: (u32, u32),
        end: (i32, i32),
        duration: Duration,
        slope_in: f32,
        slope_out: f32,
    ) -> Result<(), MyError> {
        self.impl_swipe(start.0 as i32, start.1 as i32, end.0, end.1, duration.as_millis() as u64)
    }

    fn raw_screencap(&self) -> Result<Vec<u8>, MyError> {
        self.impl_raw_screencap()
    }

    fn screencap(&self) -> Result<image::DynamicImage, MyError> {
        self.impl_screencap()
    }

    fn press_home(&self) -> Result<(), MyError> {
        self.impl_key_press(Key::Home)
    }

    fn press_esc(&self) -> Result<(), MyError> {
        self.impl_key_press(Key::Escape)
    }
}

impl PcControllerTrait for WindowsController {

    // 获取所有可见窗口
    fn get_all_windows(&self) -> Result<Vec<WindowInfo>, MyError> {
        let res = self.impl_get_all_windows()?
            .into_iter()
            .map(|w| WindowInfo {
                title: w.title,
                position: (w.rect.left, w.rect.top),
                size: ((w.rect.right - w.rect.left) as u32, (w.rect.bottom - w.rect.top) as u32),
            })
            .collect();

        Ok(res)
    }

    // // 聚焦到指定窗口
    // fn focus_window(&self, title: &str) -> Result<(), MyError> {
    //     let windows = self.impl_get_all_windows()?;
    //     for window in windows {
    //         if window.title == title {
    //             return Ok(self.impl_focus_window(window.handle)?);
    //         }
    //     }
    //     Err(MyError::S("Window not found".to_string()))
    // }

    // 模拟鼠标点击
    fn left_click(&self, x: i32, y: i32) -> Result<(), MyError> {
        self.impl_left_click(x, y)
    }

    // 模拟鼠标右键点击
    fn right_click(&self, x: i32, y: i32) -> Result<(), MyError> {
        self.impl_right_click(x, y)
    }

    // 模拟鼠标中键点击
    fn middle_click(&self, x: i32, y: i32) -> Result<(), MyError> {
        self.impl_middle_click(x, y)
    }

    // 模拟键盘按键
    fn key_click(&self, key: Key) -> Result<(), MyError> {
        self.impl_key_click(key)
    }

    // 模拟键盘按键
    fn key_press(&self, key: Key) -> Result<(), MyError> {
        self.impl_key_press(key)
    }

    // 模拟键盘释放按键
    fn key_release(&self, key: Key) -> Result<(), MyError> {
        self.impl_key_release(key)
    }

    // 模拟鼠标滑动
    fn swipe(&self, from_x: i32, from_y: i32, to_x: i32, to_y: i32, duration_ms: u64) -> Result<(), MyError> {
        self.impl_swipe(from_x, from_y, to_x, to_y, duration_ms)
    }
}

// MARK: Implementation

struct ImplWindowInfo {
    title: String,
    handle: HWND,
    rect: RECT,
}

impl WindowsController {
    
    fn true_width() -> u32 {
        unsafe {
            GetSystemMetrics(SM_CXSCREEN) as u32
        }
    }

    fn true_height() -> u32 {
        unsafe {
            GetSystemMetrics(SM_CYSCREEN) as u32
        }
    }

    // MARK: - PC Controller Impl

    fn impl_get_all_windows(&self) -> Result<Vec<ImplWindowInfo>, MyError> {
        let mut windows = Vec::new();
        
        unsafe {
            let _ = EnumWindows(Some(enum_window_proc), LPARAM(&mut windows as *mut _ as isize));
        }
        
        Ok(windows)
    }

    #[allow(dead_code)]
    fn impl_focus_window(&self, handle: HWND) -> Result<(), MyError> {
        let result = unsafe {
            SetForegroundWindow(handle).as_bool()
        };
        if result {
            Ok(())
        } else {
            Err(MyError::S("Failed to focus window".to_string()))
        }
    }

    fn impl_left_click(&self, x: i32, y: i32) -> Result<(), MyError> {
        let mut enigo = Enigo::new(&Settings::default())?;
        enigo.move_mouse(x, y, Abs).unwrap();
        enigo.button(Button::Left, Click).unwrap();
        Ok(())
    }

    fn impl_right_click(&self, x: i32, y: i32) -> Result<(), MyError> {
        let mut enigo = Enigo::new(&Settings::default())?;
        enigo.move_mouse(x, y, Abs).unwrap();
        enigo.button(Button::Right, Click).unwrap();
        Ok(())
    }

    fn impl_middle_click(&self, x: i32, y: i32) -> Result<(), MyError> {
        let mut enigo = Enigo::new(&Settings::default())?;
        enigo.move_mouse(x, y, Abs).unwrap();
        enigo.button(Button::Middle, Click).unwrap();
        Ok(())
    }

    fn impl_key_click(&self, key: Key) -> Result<(), MyError> {
        let mut enigo = Enigo::new(&Settings::default())?;
        enigo.key(key, Click).unwrap();
        Ok(())
    }

    fn impl_key_press(&self, key: Key) -> Result<(), MyError> {
        let mut enigo = Enigo::new(&Settings::default())?;
        enigo.key(key, Press).unwrap();
        Ok(())
    }

    fn impl_key_release(&self, key: Key) -> Result<(), MyError> {
        let mut enigo = Enigo::new(&Settings::default())?;
        enigo.key(key, Release).unwrap();
        Ok(())
    }

    fn impl_swipe(&self, from_x: i32, from_y: i32, to_x: i32, to_y: i32, duration_ms: u64) -> Result<(), MyError> {
        let mut enigo = Enigo::new(&Settings::default())?;
        
        // 移动到起始位置
        enigo.move_mouse(from_x, from_y, Abs).unwrap();
        
        // 计算步进值
        let steps = 20; // 将动作分为20步
        let sleep_duration = duration_ms / steps as u64;
        let x_step = (to_x - from_x) as f64 / steps as f64;
        let y_step = (to_y - from_y) as f64 / steps as f64;
        
        // 按下鼠标左键
        enigo.button(Button::Left, Press).unwrap();
        
        // 逐步移动
        for i in 1..=steps {
            let current_x = from_x as f64 + (x_step * i as f64);
            let current_y = from_y as f64 + (y_step * i as f64);
            enigo.move_mouse(current_x as i32, current_y as i32, Rel).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(sleep_duration));
        }
        
        // 释放鼠标左键
        enigo.button(Button::Left, Release).unwrap();

        Ok(())
    }

    // MARK: - Cmn Controller Impl

    fn impl_raw_screencap(&self) -> Result<Vec<u8>, MyError> {
        unsafe {
             // 获取设备上下文
            let hwnd = GetDesktopWindow();
            let hdc_screen = GetDC(Some(hwnd));
            let hdc_mem = CreateCompatibleDC(Some(hdc_screen));

            // 创建兼容位图
            let (width, height) = self.screen_size();
            let hbm_screen = CreateCompatibleBitmap(hdc_screen, width as i32, height as i32);

            // 选择位图到内存 DC
            let _old_obj = SelectObject(hdc_mem, hbm_screen.into());

            // 复制屏幕内容
            let res = BitBlt(
                hdc_mem,
                0,
                0,
                width as i32,
                height as i32,
                Some(hdc_screen),
                0,
                0,
                SRCCOPY,
            );
            if res.is_err() {
                return Err(MyError::S("Failed to capture screen".to_string()));
            }

            // 准备 BITMAPINFO 结构
            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width as i32,
                    biHeight: -(height as i32), // 负值表示从上到下的位图
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [windows::Win32::Graphics::Gdi::RGBQUAD::default(); 1],
            };

            // 获取位图数据
            let buffer_size = (width * height * 4) as usize;
            let mut buffer = vec![0u8; buffer_size];
            
            let _ = GetDIBits(
                hdc_screen,
                hbm_screen,
                0,
                height as u32,
                Some(buffer.as_mut_ptr() as _),
                &mut bmi,
                DIB_RGB_COLORS,
            );

            // 清理资源
            let _ = DeleteObject(hbm_screen.into());
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(Some(hwnd), hdc_screen);

            Ok(buffer)
        }
    }

    fn impl_screencap(&self) -> Result<image::DynamicImage, MyError> {
        let buffer = self.impl_raw_screencap()?;
        let img = image::DynamicImage::ImageRgba8(image::ImageBuffer::from_raw(self.width, self.height, buffer).unwrap());
        Ok(img)
    }


}

// 窗口枚举回调函数
extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        // 检查窗口是否可见
        if !IsWindowVisible(hwnd).as_bool() {
            return true.into();
        }

        // 获取窗口标题
        let mut title = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut title);
        if len == 0 {
            return true.into();
        }

        // 获取窗口位置和大小
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            let windows: &mut Vec<ImplWindowInfo> = &mut *(lparam.0 as *mut Vec<ImplWindowInfo>);
            let title = String::from_utf16_lossy(&title[..len as usize]);
            
            windows.push(ImplWindowInfo {
                title,
                handle: hwnd,
                rect,
            });
        }

        true.into()
    }
}

