## Task 配置

AAH 提供了一系列内置的任务，并且提供了组合任务的方法，可以通过修改 `resources/tasks.toml` 来实现自定义 ActionTask 的添加。

AAH 所提供的任务主要分为两类，他们都有自己的自定义方式：

- ActionTask：执行某些动作的任务
- <s>MatchTask：用于进行图像匹配的任务</s>



### 一、高级 ActionTask 列表

#### 

### 二、ActionTask 自定义

#### 1. 可用的原子 ActionTask

##### PressEsc

```toml
"PressEsc"
```

点击 Esc 按键

##### PressHome

```toml
"PressHome"
```

点击 Home 按键

##### Click

```toml
{
	Click = [<x>, <y>]
}
```

> 点击 `<x>`，`<y>` 坐标

##### Swipe

```toml
{
	Swipe = [[<x1>, <y1>], [<x2>, <y2>]]
}
```

> 从 `<x1>` `<y1>` 坐标滑动到 `<x2>` `<y2>` 坐标

##### ClickMatch

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

#### 2. Multi 组合

使用 Multi 可以将多个 ActionTask 组合起来顺序执行。

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

> 例：
>
> ```toml
> []
> ```

## 参考

[蚂蚁集团 | Trait 使用及实现分析 - Rust精选 (rustmagazine.github.io)](https://rustmagazine.github.io/rust_magazine_2021/chapter_4/ant_trait.html)

[Tree and Node - Programming problems in rust (ratulb.github.io)](https://ratulb.github.io/programming_problems_in_rust/binary_search_tree/tree_and_node.html)

