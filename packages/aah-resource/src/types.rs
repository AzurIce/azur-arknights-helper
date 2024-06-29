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
    use super::{HeightType, Level};

    #[test]
    fn ser() {
        let height_type = HeightType::HightLand;
        let json = serde_json::to_string_pretty(&height_type).unwrap();
        println!("{}", json);
    }

    #[test]
    fn f() {
        let level = r#"
{
    "name": "坍塌",
    "code": "0-1",
    "levelId": "obt/main/level_main_00-01",
    "stageId": "main_00-01",
    "width": 9,
    "height": 6,
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
        }
      ],
      [
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
          "heightType": 1,
          "buildableType": 2,
          "tileKey": "tile_wall",
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
          "heightType": 1,
          "buildableType": 2,
          "tileKey": "tile_wall",
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
        -4.81,
        -7.76
      ],
      [
        0.5975098586953793,
        -5.31,
        -8.642108163374733
      ]
    ]
}"#;
        let level = serde_json::from_str::<Level>(&level).unwrap();
        println!("{level:?}")
    }
}
