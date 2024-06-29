use crate::utils::{camera_euler_angles_xyz, world_to_screen};
use nalgebra as na;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

pub struct TilePos {
    pub y: u32,
    pub x: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Level {
    pub name: String,
    pub code: String,
    pub level_id: String,
    pub stage_id: String,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Vec<Tile>>, // height[width]
    pub view: [[f32; 3]; 2],
}

impl Level {
    // 计算相机位置
    pub fn camera_pos(&self, side: bool, width: f32, height: f32) -> na::Vector3<f32> {
        let (x, y, z) = if side {
            (self.view[1][0], self.view[1][1], self.view[1][2])
        } else {
            (self.view[0][0], self.view[0][1], self.view[0][2])
        };

        const FROM_RATIO: f32 = 9.0 / 16.0;
        const TO_RATIO: f32 = 3.0 / 4.0;
        let ratio = height / width;
        let t = (FROM_RATIO - ratio) / (FROM_RATIO - TO_RATIO);
        let pos_adj = na::Vector3::new(-1.4 * t, -2.8 * t, 0.0);

        na::Vector3::new(x + pos_adj.x, y + pos_adj.y, z + pos_adj.z)
    }

    pub fn get_tile(&self, y: usize, x: usize) -> &Tile {
        self.tiles.get(y).unwrap().get(x).unwrap()
    }

    // 计算 `tile_pos` 中心点在世界坐标中的位置
    pub fn tile_world_pos(&self, y: u32, x: u32) -> na::Vector3<f32> {
        let tile = self.get_tile(y as usize, x as usize);
        let z = match tile.height_type {
            HeightType::HightLand => 0.0,
            HeightType::LowLand => -0.4,
        };
        na::Vector3::new(
            x as f32 - (self.width as i32 - 1) as f32 / 2.0,
            (self.height - 1) as f32 / 2.0 - y as f32,
            z,
        )
    }

    /// 计算 `tile_pos` 中心点在屏幕中的位置
    pub fn calc_tile_screen_pos(&self, y: u32, x: u32, side: bool) -> (f32, f32) {
        let width = 1920.0;
        let height = 1080.0;
        let camera_pos = self.camera_pos(side, width, height);
        let camera_euler = camera_euler_angles_xyz(side);
        let world_pos = self.tile_world_pos(y, x);
        world_to_screen(camera_pos, camera_euler, world_pos, width, height)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tile {
    pub height_type: HeightType,
    pub buildable_type: BuildableType,
    pub tile_key: String,
    pub is_start: bool,
    pub is_end: bool,
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
/// 高台/地面类型
pub enum HeightType {
    /// 高台
    HightLand = 0,
    /// 地面
    LowLand = 1,
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
/// 部署类型
pub enum BuildableType {
    /// 近战
    Melee = 0,
    /// 远程
    Ranged = 1,
    /// 不可部署
    None = 2,
}

#[cfg(test)]
mod test {
    use image::{DynamicImage, GenericImage, Rgba};

    use super::{HeightType, Level};

    fn draw_box(
        image: &mut DynamicImage,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        rgba_u8: [u8; 4],
    ) {
        for dx in 0..width {
            let px = x + dx as i32;
            let py1 = y;
            let py2 = y + height as i32;

            if px >= 0 && py1 >= 0 && px < image.width() as i32 && py2 < image.height() as i32 {
                image.put_pixel(px as u32, py1 as u32, Rgba(rgba_u8));
            }
            if px >= 0 && py2 >= 0 && px < image.width() as i32 && py2 < image.height() as i32 {
                image.put_pixel(px as u32, py2 as u32, Rgba(rgba_u8));
            }
        }

        for dy in 0..height {
            let py = y + dy as i32;
            let px1 = x;
            let px2 = x + width as i32;

            if px1 >= 0 && py >= 0 && px1 < image.width() as i32 && py < image.height() as i32 {
                image.put_pixel(px1 as u32, py as u32, Rgba(rgba_u8));
            }
            if px2 >= 0 && py >= 0 && px2 < image.width() as i32 && py < image.height() as i32 {
                image.put_pixel(px2 as u32, py as u32, Rgba(rgba_u8));
            }
        }
        // for dx in 0..width {
        //     for dy in 0..=height {
        //         let px = x + dx as i32;
        //         let py = y + dy as i32;
        //         // 边界检查
        //         if px >= 0 && py >= 0 && px < image.width() as i32 && py < image.height() as i32 {
        //             image.put_pixel(px as u32, py as u32, Rgba(rgba_u8));
        //         }
        //     }
        // }
    }

    fn draw_tile_centers(image: &mut DynamicImage, level: &Level) {
        for i in 0..level.height {
            for j in 0..level.width {
                let tile_world_pos = level.tile_world_pos(i, j);
                let tile_screen_pos = level.calc_tile_screen_pos(i, j, false);
                println!(
                    "({i}, {j}): world {:?}, screen {:?}",
                    tile_world_pos, tile_screen_pos
                );
                let (x, y) = (
                    tile_screen_pos.0.round() as u32,
                    tile_screen_pos.1.round() as u32,
                );
                image.put_pixel(x, y, Rgba([0, 255, 0, 255]))
            }
        }
    }

    fn draw_direction_box(image: &mut DynamicImage, level: &Level) {
        for i in 0..level.height {
            for j in 0..level.width {
                let tile_world_pos = level.tile_world_pos(i, j);
                let tile_screen_pos = level.calc_tile_screen_pos(i, j, false);
                println!(
                    "({i}, {j}): world {:?}, screen {:?}",
                    tile_world_pos, tile_screen_pos
                );
                let (x, y) = (
                    tile_screen_pos.0.round() as u32,
                    tile_screen_pos.1.round() as u32,
                );
                let x = x as i32 - 48;
                let y = y as i32 - 48;
                draw_box(image, x, y, 96, 96, [255, 0, 0, 255]);
            }
        }
    }

    fn crop_direction_box(image: &DynamicImage, level: &Level, y: u32, x: u32) -> image::DynamicImage {
        let tile_screen_pos = level.calc_tile_screen_pos(y, x, false);
        let (x, y) = (
            tile_screen_pos.0.round() as u32 - 48,
            tile_screen_pos.1.round() as u32 - 48,
        );
        image.crop_imm(x, y, 96, 96)
    }

    #[test]
    fn cut() {
        let level = serde_json::from_str::<Level>(&LS_6).unwrap();
        let image = image::open("./assets/LS-6_0.png").unwrap();
        crop_direction_box(&image, &level, 2, 3).save("./assets/output/LS-6_0_left0.png").unwrap();
        crop_direction_box(&image, &level, 3, 3).save("./assets/output/LS-6_0_right0.png").unwrap();
        crop_direction_box(&image, &level, 4, 3).save("./assets/output/LS-6_0_left1.png").unwrap();
        crop_direction_box(&image, &level, 5, 3).save("./assets/output/LS-6_0_up0.png").unwrap();
        crop_direction_box(&image, &level, 3, 5).save("./assets/output/LS-6_0_right1.png").unwrap();
        crop_direction_box(&image, &level, 4, 5).save("./assets/output/LS-6_0_right2.png").unwrap();
        crop_direction_box(&image, &level, 5, 5).save("./assets/output/LS-6_0_right3.png").unwrap();
        crop_direction_box(&image, &level, 4, 6).save("./assets/output/LS-6_0_left2.png").unwrap();

        let image = image::open("./assets/LS-6_1.png").unwrap();
        crop_direction_box(&image, &level, 2, 3).save("./assets/output/LS-6_1_left0.png").unwrap();
        crop_direction_box(&image, &level, 3, 3).save("./assets/output/LS-6_1_right0.png").unwrap();
        crop_direction_box(&image, &level, 4, 3).save("./assets/output/LS-6_1_left1.png").unwrap();
        crop_direction_box(&image, &level, 5, 3).save("./assets/output/LS-6_1_up0.png").unwrap();
        crop_direction_box(&image, &level, 3, 5).save("./assets/output/LS-6_1_right1.png").unwrap();
        crop_direction_box(&image, &level, 4, 5).save("./assets/output/LS-6_1_right2.png").unwrap();
        crop_direction_box(&image, &level, 5, 5).save("./assets/output/LS-6_1_right3.png").unwrap();
        crop_direction_box(&image, &level, 4, 6).save("./assets/output/LS-6_1_left2.png").unwrap();
    }

    #[test]
    fn fooo() {
        let level = serde_json::from_str::<Level>(&LS_6).unwrap();
        let mut image = image::open("./assets/in_battle.png").unwrap();

        draw_tile_centers(&mut image, &level);
        draw_direction_box(&mut image, &level);

        image.save("./assets/in_battle_drawed.png").unwrap();
    }

    #[test]
    fn ser() {
        let height_type = HeightType::HightLand;
        let json = serde_json::to_string_pretty(&height_type).unwrap();
        println!("{}", json);
    }

    #[test]
    fn f() {
        let level = serde_json::from_str::<Level>(&LS_6).unwrap();
        println!("{level:?}")
    }

    const LS_6: &str = r#"
    {
    "name": "运动战演习",
    "code": "LS-6",
    "levelId": "obt/weekly/level_weekly_killcost_6",
    "stageId": "wk_kc_6",
    "width": 9,
    "height": 8,
    "tiles": [
      [
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        }
      ],
      [
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_flystart",
          "isStart": true,
          "isEnd": false
        }
      ],
      [
        {
          "heightType": 0,
          "buildableType": 0,
          "tileKey": "tile_start",
          "isStart": true,
          "isEnd": false
        },
        {
          "heightType": 0,
          "buildableType": 1,
          "tileKey": "tile_road",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 0,
          "buildableType": 1,
          "tileKey": "tile_road",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 0,
          "buildableType": 1,
          "tileKey": "tile_road",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 0,
          "buildableType": 0,
          "tileKey": "tile_end",
          "isStart": false,
          "isEnd": true
        },
        {
          "heightType": 1,
          "buildableType": 2,
          "tileKey": "tile_wall",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 2,
          "tileKey": "tile_wall",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        }
      ],
      [
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 2,
          "tileKey": "tile_wall",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 2,
          "tileKey": "tile_wall",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 0,
          "buildableType": 0,
          "tileKey": "tile_end",
          "isStart": false,
          "isEnd": true
        },
        {
          "heightType": 0,
          "buildableType": 1,
          "tileKey": "tile_road",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 0,
          "buildableType": 1,
          "tileKey": "tile_road",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 0,
          "buildableType": 1,
          "tileKey": "tile_road",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 0,
          "buildableType": 0,
          "tileKey": "tile_start",
          "isStart": true,
          "isEnd": false
        }
      ],
      [
        {
          "heightType": 0,
          "buildableType": 0,
          "tileKey": "tile_start",
          "isStart": true,
          "isEnd": false
        },
        {
          "heightType": 0,
          "buildableType": 1,
          "tileKey": "tile_road",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 0,
          "buildableType": 1,
          "tileKey": "tile_road",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 0,
          "buildableType": 1,
          "tileKey": "tile_road",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 0,
          "buildableType": 0,
          "tileKey": "tile_end",
          "isStart": false,
          "isEnd": true
        },
        {
          "heightType": 1,
          "buildableType": 2,
          "tileKey": "tile_wall",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 2,
          "tileKey": "tile_wall",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        }
      ],
      [
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 2,
          "tileKey": "tile_wall",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 2,
          "tileKey": "tile_wall",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 0,
          "buildableType": 0,
          "tileKey": "tile_end",
          "isStart": false,
          "isEnd": true
        },
        {
          "heightType": 0,
          "buildableType": 1,
          "tileKey": "tile_road",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 0,
          "buildableType": 1,
          "tileKey": "tile_road",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 0,
          "buildableType": 1,
          "tileKey": "tile_road",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 0,
          "buildableType": 0,
          "tileKey": "tile_start",
          "isStart": true,
          "isEnd": false
        }
      ],
      [
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_flystart",
          "isStart": true,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        }
      ],
      [
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": true,
          "isEnd": true
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        },
        {
          "heightType": 1,
          "buildableType": 0,
          "tileKey": "tile_forbidden",
          "isStart": false,
          "isEnd": false
        }
      ]
    ],
    "view": [
      [
        0.0,
        -5.6,
        -8.9
      ],
      [
        0.79546878123568,
        -6.1,
        -9.764789001808651
      ]
    ]
  }"#;
}
