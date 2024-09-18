# AAH~（AzurArknightsHelper）


<s>本来想拿 MaaCore 在 Rust 里搓点有意思的东西，但是 cpp 的构建过程实在是太烦人了，所以决定纯 Rust 重写。</s>

- [ ] 实现 TemplateMatching

## 如何使用

```
Usage: aah [OPTIONS] [COMMAND]

Commands:
  task     run task
  copilot  run copilot
  help     Print this message or the help of the given subcommand(s)

Options:
  -s, --serial-number <SERIAL_NUMBER>  The serial number of the target device, default: 127.0.0.1:16384
  -h, --help                           Print help
  -V, --version                        Print version
```

目前，aah 仅支持 1920x1080 的屏幕，建议使用 MUMU 模拟器，并在设置中调节分辨率为 1920x1080。

### 1. Task 执行

> 后续会实现自定义 Task 的功能

aah 提供了一些自带的任务：

- `start_up`：开始唤醒
- `award`：领取奖励





## Task 配置

AAH 提供了一系列内置的任务，并且提供了组合任务的方法，可以通过修改 `resources/tasks.toml` 或添加额外的 `resources/tasks/<task_name>.toml` 来实现自定义任务的添加（其实内置的任务也有很多是通过 `toml` 声明的，所以可以参考参考）。

以下是内置的的 `award` 任务例子：

```toml
# resources/tasks.toml
[award]
[award.Multi]
fail_fast = false
tasks = [
    { NavigateIn = "mission" },
    { ByName = { name = "press_collect_all_award", wrapper = { delay = 0.5, retry = 1 } } },
    { ByName = { name = "confirm", wrapper = { delay = 0.5, retry = 1 } } },
    { ActionClickMatch = {
    	match_task = { type = "Template", template = "award_2.png" },
    	wrapper = { delay = 0.5, retry = 1 } }
    },
    { ByName = { name = "press_collect_all_award" } },
    { ByName = { name = "confirm", wrapper = { delay = 0.5, retry = 1 } } },
    { NavigateOut = "mission", wrapper = { delay = 0.5, retry = 1 } },
]

```

也可以选择在 `resources/tasks/` 下创建一个 `award.toml`：

```toml
# resources/tasks/award.toml
[Multi]
fail_fast = false
tasks = [
    { NavigateIn = "mission" },
    { ByName = { name = "press_collect_all_award", wrapper = { delay = 0.5, retry = 1 } } },
    { ByName = { name = "confirm", wrapper = { delay = 0.5, retry = 1 } } },
    { ActionClickMatch = {
    	match_task = { type = "Template", template = "award_2.png" },
    	wrapper = { delay = 0.5, retry = 1 } }
    },
    { ByName = { name = "press_collect_all_award" } },
    { ByName = { name = "confirm", wrapper = { delay = 0.5, retry = 1 } } },
    { NavigateOut = "mission", wrapper = { delay = 0.5, retry = 1 } },
]

```

其中的各任务具体解释见下

### 一、高级 Task 列表

高级 Task，是一系列内置的具有较为完整功能的任务。

#### start_up

```toml
{ ByName = { name = "award" } }
```

开始唤醒，到达主页

#### award

```toml
{ ByName = { name = "award" } }
```

从主页开始，领取所有奖励，再返回主页

### 二、工具 Task 列表

工具 Task，是一系列较为常用的子 Task。

- 一些 `ActionClickMatch` 的 alias，暂略：
  - back
  - click_collect_all_award

### 三、内置 Task 列表

内置 Task，就是通过非 `toml` 实现的 Task。

例如一个比较基础的 ActionClick：

```toml
{ ActionClick = { x = <x>, y = <y> } }
```

此外可以为任务添加 TaskWrapper，用于对任务进行通用的配置，不同的 Task 支持不同的 TaskWrapper（详情见具体 Task 及 TaskWrapper 描述）。

例如大部分任务都支持的 GenericTaskWrapper：

```toml
{
	ActionClick = { x = <x>, y = <y>, wrapper = {
		delay = 0.5
		retry = 3
		repeat = 2
	} }
}
```

#### 1. 基础 ActionTask

均支持 `GenericTaskWrapper`。

##### ActionPressEsc

```toml
{ ActionPressEsc = {} }
```

点击 Esc 按键

##### ActionPressHome

```toml
{ ActionPressHome = {} }
```

点击 Home 按键

##### ActionClick

```toml
{
	ActionClick = { x = <x>, y = <y> }
}
```

> 点击 `<x>`，`<y>` 坐标（整数）

##### ActionSwipe

```toml
{
	ActionSwipe = { p1 = [<x1>, <y1>], p2 = [<x2>, <y2>], duration = 1.0 }
}
```

> 从 `<x1>` `<y1>` 坐标滑动到 `<x2>` `<y2>` 坐标（整数）持续时间 `duration` 秒

##### ActionClickMatch

###### > Template

```toml
{
	ActionClickMatch = {
		match_task = {
            type = "Template"
            template = "image.png" # 位于 resource/template/ 下的文件
		}
	}
}
```

###### > Ocr（暂未实现，匹配结果永远是失败）

```toml
{
	ClickMatch = {
		match_task = {
            type = "Ocr"
            template = "启动!" # 查找的目标串
		}
	}
}
```

#### 2. NavigateIn / NavigateOut

从主页进入到某一页面/从某一页面退出到主页

```toml
{ NavigateIn = "page_name" }
```

```toml
{ NavigateOut = "page_name" }
```

其中 `page_name` 以及具体导航方式由 `navigates.toml` 配置，详情见 [四、Navigate 定义][]

#### 3. Multi 组合

使用 Multi 可以将多个 Task 组合起来顺序执行。

fail_fast 用于配置遇到错误时是否直接退出

```toml
[Multi]
fail_fast = true,
tasks = [
    { ActionPressEsc = {} },
    { NavigateIn = "name" },
    { ActionPressHome = {} },
    { ActionClick = { x = 0, y = 0, wrapper = { delay = 0.0, retry = 0, repeat = 1 } } },
    { ActionSwipe = { p1 = [
    0,
    0,
], p2 = [
    200,
    0,
], duration = 1.0 } },
    "task_name",
]
```

#### 4. ByName

通过任务名来引用任务

支持 `GenericTaskWrapper`

```toml
{ ByName = { name = "page_name" } }
```

## 四、Navigate 定义

NavigateTask 中所使用的 page_name 及对应的详细导航方式均由 `resources/navigates.toml` 或 `resources/navigates/<page_name>.toml` 定义。

下面是一个任务界面的 NavigateTask 的配置案例：

```toml
# resources/navigates.toml
[mission]
enter_task = { ActionClickMatch = {
	match_task = { type = "Template", template = "EnterMissionMistCity.png" }
} }
exit_task = { ByName = { name = "back" } }
```

- `enter_task` 表示从主页进入的方法，也就是 NavigateIn 执行的任务。
- `exit_task` 表示退出到主页的方法，也就是 NavigateOut 执行的任务。

## 参考

[蚂蚁集团 | Trait 使用及实现分析 - Rust精选 (rustmagazine.github.io)](https://rustmagazine.github.io/rust_magazine_2021/chapter_4/ant_trait.html)

[Tree and Node - Programming problems in rust (ratulb.github.io)](https://ratulb.github.io/programming_problems_in_rust/binary_search_tree/tree_and_node.html)

