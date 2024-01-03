# AAH~（AzurArknightsHelper）

<s>本来想拿 MaaCore 在 Rust 里搓点有意思的东西，但是 cpp 的构建过程实在是太烦人了，所以决定纯 Rust 重写。</s>

## 如何使用

咕咕咕，现在 main 还是空的（），正在狠狠搭框子。

## Task 配置

AAH 提供了一系列内置的任务，并且提供了组合任务的方法，可以通过修改 `resources/tasks.toml` 或添加额外的 `resources/tasks/<task_name>.toml` 来实现自定义任务的添加（其实内置的任务也有很多是通过 `toml` 声明的，所以可以参考参考）。

以下是一个自定义的 `award` 任务例子：

```toml
# resources/tasks.toml
[award]
Multi = [
    { NavigateIn = "mission" },
    "press_collect_all_award",
    { ActionClickMatch = { type = "Template", template = "award_2.png" } },
    "press_collect_all_award",
    { ActionClick = [100, 100]},
    { NavigateOut = "mission" },
]
```

也可以选择在 `resources/tasks/` 下创建一个 `award.toml`：

```toml
# resources/tasks/award.toml
Multi = [
    { NavigateIn = "mission" },
    "press_collect_all_award",
    { ActionClickMatch = { type = "Template", template = "award_2.png" } },
    "press_collect_all_award",
    { ActionClick = [100, 100]},
    { NavigateOut = "mission" },
]
```

### 一、高级 Task 列表

高级 Task，是一系列内置的具有较为完整功能的任务。

#### award

```toml
"award"
```

从主页开始，领取所有奖励，再返回主页

### 二、工具 Task 列表

工具 Task，是一系列较为常用的子 Task。

- 一些 `ActionClickMatch` 的 alias，暂略：
  - back
  - click_collect_all_award



### 三、内置 Task 列表

内置 Task，就是通过非 `toml` 实现的 Task。

##### ActionPressEsc

```toml
"PressEsc"
```

点击 Esc 按键

##### ActionPressHome

```toml
"PressHome"
```

点击 Home 按键

##### ActionClick

```toml
{
	Click = [<x>, <y>]
}
```

> 点击 `<x>`，`<y>` 坐标（整数）

##### ActionSwipe

```toml
{
	Swipe = [[<x1>, <y1>], [<x2>, <y2>]]
}
```

> 从 `<x1>` `<y1>` 坐标滑动到 `<x2>` `<y2>` 坐标（整数）

##### ActionClickMatch

###### > Template

```toml
{
	ClickMatch = {
		type = "Template"
		template = "image.png" # 位于 resource/template/ 下的文件
	}
}
```

###### > Ocr（暂未实现，匹配结果永远是失败）

```toml
{
	ClickMatch = {
		type = "Ocr"
		template = "启动!" # 查找的目标串
	}
}
```

#### NavigateIn / NavigateOut

从主页进入到某一页面/从某一页面退出到主页

```toml
{ NavigateIn = "page_name" }
```

```toml
{ NavigateOut = "page_name" }
```

其中 `page_name` 以及具体导航方式由 `navigates.toml` 配置，详情见 四、Navigate 定义

#### 2. Multi 组合

使用 Multi 可以将多个 ActionTask 组合起来顺序执行（如果某一任务失败会继续执行后续任务）。

```toml
Multi = [
    "ActionPressEsc",
    "ActionPressHome",
    { ActionClick = [ 0, 0 ] },
    { ActionSwipe = [
        [ 0, 0 ],
        [ 200, 0 ],
	] },
    "task_name", # 可以通过任务名称来引用自定义的任务
]
```

## 四、Navigate 定义

NavigateTask 中所使用的 page_name 及对应的详细导航方式均由 `resources/navigates.toml` 或 `resources/navigates/<page_name>.toml` 定义。

下面是一个任务界面的 NavigateTask 的配置案例：

```toml
# resources/navigates.toml
[mission]
enter_task = {
	ActionClickMatch = { type = "Template", template = "EnterMissionMistCity.png"}
}
exit_task = "back"
```

- `enter_task` 表示从主页进入的方法，也就是 NavigateIn 执行的任务。
- `exit_task` 表示退出到主页的方法，也就是 NavigateOut 执行的任务。

## 参考

[蚂蚁集团 | Trait 使用及实现分析 - Rust精选 (rustmagazine.github.io)](https://rustmagazine.github.io/rust_magazine_2021/chapter_4/ant_trait.html)

[Tree and Node - Programming problems in rust (ratulb.github.io)](https://ratulb.github.io/programming_problems_in_rust/binary_search_tree/tree_and_node.html)

