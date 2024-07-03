# Battle



## Copilot

## BattleProcessTask

1. `calc_tiels_info`

    > https://github.com/MaaAssistantArknights/MaaAssistantArknights/blob/dev/src/MaaCore/Task/BattleHelper.cpp#L57-L76

    1. 初始化 `m_map_data`

        ```cpp
        m_map_data = TilePack::find_level(stage_name).value_or(Map::Level {});
        ```

    2. 计算地块信息

        > https://github.com/MaaAssistantArknights/MaaAssistantArknights/blob/37aff294435a63d448095e20dedd3784a170b221/src/MaaCore/Config/Miscellaneous/TilePack.h#L94-L103
        >
        > https://github.com/MaaAssistantArknights/MaaAssistantArknights/blob/37aff294435a63d448095e20dedd3784a170b221/src/MaaCore/Config/Miscellaneous/TilePack.cpp#L86-L121

        ```cpp
        auto calc_result = TilePack::calc(stage_name, shift_x, shift_y);
        ```

        - `m_normal_tile_info`
        - `m_side_tile_info`
        - `m_retreat_button_pos`
        - `m_skill_button_pos`

---

## 部署分析

> https://github.com/MaaAssistantArknights/MaaAssistantArknights/blob/dev/src/MaaCore/Vision/Battle/BattlefieldMatcher.cpp#L71

