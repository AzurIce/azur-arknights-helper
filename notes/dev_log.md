## 有关设备控制

首先基于 `adb` 命令实现了 `AdbController`，难以实现复杂的触控操作。

之后引入了 [DeviceFarmer/minitouch](https://github.com/DeviceFarmer/minitouch) 与 `adb` 配合，实现了 `MiniTouchController`。

## 有关游戏世界的信息处理

https://github.com/yuanyan3060/ArknightsGameResource/tree/main

通过解包游戏数据可以获得很多可以利用的数据。

- `levels.json`：



- [ ] 分析 MAA：
    - [ ] 编队识别
    - [x] 待部署区识别
    - [ ] 场上干员识别
        - [x] 地块到像素坐标转换
- [ ] 作业文件
    - [ ] 执行作业