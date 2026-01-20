## 新的设计

所有 adb、controller、cv 等内容都被提取到了 https://github.com/AzurIce/auto-play 作为一个基础库。

也就是说，原先的 Task、Resource、Template 等内容现在全都在 azur-arknights-helper 中实现。

### “蓝图”系统

其实所谓“蓝图”系统只是对一个可序列化结构的可视化编辑，而对于一个任务执行系统（Task）来说，
其实更“线性”一些，类似 Github Action。（目前还没有想好要不要引入控制流、循环，还是纯粹类似 workflow 一样的线性结构）

不过大致都是一样：
- 需要一种序列化方式来表示蓝图
- 需要一个 GUI 界面来编辑蓝图

目前的想法依旧是基于 toml 格式。

### Rhai

之前定义 Task 的方式仅有编写 toml 并使用已有的 Action，在表达能力上有一定的局限性。

可以将核心的 Aah 结构暴露给 Rhai，来允许使用 Rhai 编写脚本来自定义 Action。

### Task
