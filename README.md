# AAH~（AzurArknightsHelper）


<s>本来想拿 MaaCore 在 Rust 里搓点有意思的东西，但是 cpp 的构建过程实在是太烦人了，所以决定纯 Rust 重写。</s>

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

## Task 配置

AAH 提供了一系列内置的任务，并且提供了组合任务的方法，可以通过添加额外的 `resources/tasks/<task_name>.toml` 来实现自定义任务的添加（其实内置的任务也有很多是通过 `toml` 声明的，所以可以参考参考）。

以下是内置的的 `award` 任务例子：

```toml
# resources/tasks/award.toml
name = "award"

[[steps]]

[steps.action]
ByName = "enter_mission"

[[steps]]
delay_sec = 0.5
skip_if_failed = true
retry = 1

[steps.action.ClickMatchTemplate]
template = "mission-week_collect-all.png"

# ...
```

每一个任务有一个 `name` 字段作为唯一标识符，与其文件名一致。

一个任务由若干个 `step` 构成，每个 `step` 有一系列的参数可以控制其执行策略：
- `delay_sec`：在执行此步前的延迟秒数。（默认：0）
- `skip_if_failed`：如果失败是否跳过，不跳过即 fail-fast。（默认：false）
- `repeat`：重复次数。（默认：0）
- `retry`：每次重试次数，< 0 表示一直重试直至成功。（默认：0）

每个 `step` 中，定义具体执行任务的是 `action` 字段。

### 可用 Action

#### ByName

```toml
[[steps]]

[steps.action]
ByName = "enter_mission"
```

将对应名称的 Task 作为一个 Action 执行。

#### PressEsc

```toml
[[steps]]

[steps.action.PressEsc]
```

点击 Esc 按键

#### PressHome


```toml
[[steps]]

[steps.action.PressHome]
```

点击 Home 按键

#### Click

```toml
[[steps]]

[steps.action.Click]
x = <x>
y = <y>
```

点击 `<x>`，`<y>` 坐标（整数）

#### Swipe

```toml
[[steps]]

[steps.action.Click]
p1 = { x = 400, y = 400 }
p2 = { x = 600, y = 600 }
duration = 0.5
```

从 `<x1>` `<y1>` 坐标滑动到 `<x2>` `<y2>` 坐标（整数）持续时间 `duration` 秒

#### ClickMatch

###### > Template

```toml
[[steps]]

[steps.action.ClickMatchTemplate]
template = "confirm.png"
```

匹配指定模板图片并点击，模板图片需要置于 `resources/templates/1920x1080` 目录下。



## 致谢
- https://github.com/MaaAssistantArknights/MaaAssistantArknights
- https://github.com/DeviceFarmer/minitouch