pub use display_info::DisplayInfo;

mod image;
pub use image::Image;

#[cfg(target_os = "macos")]
mod darwin;
#[cfg(target_os = "macos")]
use darwin::*;

#[cfg(target_os = "windows")]
mod win32;
#[cfg(target_os = "windows")]
use win32::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux::*;

#[derive(Debug, Clone, Copy)]
pub struct Screen {
  pub display_info: DisplayInfo,
}

impl Screen {
  pub fn new(display_info: &DisplayInfo) -> Self {
    Screen {
      display_info: *display_info,
    }
  }

  pub fn all() -> Option<Vec<Screen>> {
    let screens = DisplayInfo::all()?.iter().map(Screen::new).collect();
    Some(screens)
  }

  pub fn from_point(x: i32, y: i32) -> Option<Screen> {
    let display_info = DisplayInfo::from_point(x, y)?;
    Some(Screen::new(&display_info))
  }

  pub fn capture_all_screens() -> Option<Image> {
    let screens = Screen::all().unwrap();

    let mut x_min = 0;
    let mut x_max = 0;

    let mut y_min = 0;
    let mut y_max = 0;

    // Get the full image size.
    for screen in &screens {
      let di = screen.display_info;
      println!("´DI {di:?}");

      if x_min > di.x {
        x_min = di.x;
      }

      if x_max < di.width as i32 + di.x {
        x_max = di.width as i32 + di.x;
      }

      if y_min > di.y {
        y_min = di.y;
      }

      if y_max < di.height as i32 + di.y {
        y_max = di.height as i32 + di.y;
      }
    }

    let tot_width = x_max - x_min;
    let tot_height = y_max - y_min;

    let mut tot_buffer: Vec<u8> = Vec::new();
    tot_buffer.resize((tot_height * tot_width * 4) as usize, 0);

    for screen in screens {
      let di = screen.display_info;

      let y_norm = di.y + y_min.abs();
      let x_norm = di.x + x_min.abs();

      if let Some(data) = capture_screen_raw(&di) {
        let data_width = 4 * tot_width;
        for i in 0..di.height as i32 {
          let from = (y_norm + i) * data_width + x_norm * 4;
          let to = (y_norm + i) * data_width + x_norm * 4 + di.width as i32 * 4;
          let slice = &mut tot_buffer[from as usize..to as usize];

          slice.copy_from_slice(
            &data[(i as usize * di.width as usize * 4)
              ..(i as usize * di.width as usize * 4 + di.width as usize * 4)],
          );
        }
      }
    }

    Image::from_bgra(tot_width as u32, tot_height as u32, tot_buffer).ok()
  }

  pub fn capture(&self) -> Option<Image> {
    capture_screen(&self.display_info)
  }

  /**
   * 截取指定区域
   * 区域x,y为相对于当前屏幕的x,y坐标
   */
  pub fn capture_area(&self, x: i32, y: i32, width: u32, height: u32) -> Option<Image> {
    let display_info = self.display_info;
    let screen_x2 = display_info.x + display_info.width as i32;
    let screen_y2 = display_info.y + display_info.height as i32;

    let mut x1 = x + display_info.x;
    let mut y1 = y + display_info.y;
    let mut x2 = x1 + width as i32;
    let mut y2 = y1 + height as i32;

    // x y 必须在屏幕范围内
    if x1 < display_info.x {
      x1 = display_info.x;
    } else if x1 > screen_x2 {
      x1 = screen_x2
    }

    if y1 < display_info.y {
      y1 = display_info.y;
    } else if y1 > screen_y2 {
      y1 = screen_y2;
    }

    if x2 > screen_x2 {
      x2 = screen_x2;
    }

    if y2 > screen_y2 {
      y2 = screen_y2;
    }

    if x1 >= x2 || y1 >= y2 {
      return None;
    }

    capture_screen_area(
      &display_info,
      x1 - display_info.x,
      y1 - display_info.y,
      (x2 - x1) as u32,
      (y2 - y1) as u32,
    )
  }
}
