





https://github.com/MaaAssistantArknights/MaaAssistantArknights/tree/dev/resource/Arknights-Tile-Pos

MAA 的资源更新 Github Action：

https://github.com/MaaAssistantArknights/MaaAssistantArknights/actions/runs/9708784579/workflow

数据来源：

https://github.com/yuanyan3060/ArknightsGameResource/blob/main/gamedata/levels/activities/a001/level_a001_01.json

关卡数据更新具体代码：

https://github.com/MaaAssistantArknights/MaaAssistantArknights/blob/dev/tools/ResourceUpdater/main.cpp#L781-L823



计算映射代码：

https://github.com/MaaAssistantArknights/MaaAssistantArknights/blob/dev/src/MaaCore/Task/BattleHelper.cpp#L57-L76



https://github.com/MaaAssistantArknights/MaaAssistantArknights/blob/37aff294435a63d448095e20dedd3784a170b221/src/MaaCore/Config/Miscellaneous/TilePack.cpp#L86-L121



https://github.com/MaaAssistantArknights/MaaAssistantArknights/blob/37aff294435a63d448095e20dedd3784a170b221/3rdparty/include/Arknights-Tile-Pos/TileCalc2.hpp







相机位置：

### 相机位置

```cpp
inline vec3d camera_pos(const Level& level, bool side = false, int width = 1280, int height = 720)
{
    const auto [x, y, z] = level.view[side ? 1 : 0];

    static constexpr double fromRatio = 9. / 16;
    static constexpr double toRatio = 3. / 4;
    const double ratio = static_cast<double>(height) / width;
    const double t = (fromRatio - ratio) / (fromRatio - toRatio);
    const vec3d pos_adj = { -1.4 * t, -2.8 * t, 0 };
    return { x + pos_adj[0], y + pos_adj[1], z + pos_adj[2] };
}
```

### 相机欧拉角

```cpp
inline vec3d camera_euler_angles_yxz(const Level& /*level*/, bool side = false)
{
    if (side) {
        return { 10 * degree, 30 * degree, 0 };
    }

    return { 0, 30 * degree, 0 };
}
```

### 相机矩阵

```cpp
inline matrix4x4 camera_matrix_from_trans(
    const vec3d& pos,
    const vec3d& euler,
    double ratio,
    double fov_2_y = 20 * degree,
    double far = 1000,
    double near = 0.3)
{
    const double cos_y = std::cos(euler[0]);
    const double sin_y = std::sin(euler[0]);
    const double cos_x = std::cos(euler[1]);
    const double sin_x = std::sin(euler[1]);
    const double tan_f = std::tan(fov_2_y);

    const matrix4x4 translate = {
        1, 0, 0, -pos[0], //
        0, 1, 0, -pos[1], //
        0, 0, 1, -pos[2], //
        0, 0, 0, 1,
    };
    const matrix4x4 matrixY = {
        cos_y,  0, sin_y, 0, //
        0,      1, 0,     0, //
        -sin_y, 0, cos_y, 0, //
        0,      0, 0,     1,
    };
    const matrix4x4 matrixX = {
        1, 0,      0,      0, //
        0, cos_x,  -sin_x, 0, //
        0, -sin_x, -cos_x, 0, //
        0, 0,      0,      1,
    };
    const matrix4x4 proj = {
        // clang-format off
        ratio / tan_f,  0,         0, 0,
        0,              1 / tan_f, 0, 0,
        0,              0,         -(far + near) / (far - near), -(far * near * 2) / (far - near),
        0,              0,         -1, 0,
        // clang-format on
    };

    return proj * matrixX * matrixY * translate;
}
```

