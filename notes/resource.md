aah 的资源文件包含：

- （体积较大）由游戏素材处理而来的资源（如匹配模板、关卡数据）
- 提供的 Task 文件
- 提供的 Copilot 文件

考虑到资源文件整体的大小：

- 为了不让可执行文件过大，资源不应该直接打包到可执行文件中，而应该运行时下载并加载。

- 为了减少内存使用，还是应当 lazy load，而非一股脑全部加载到内存中



资源文件与代码同仓库，运行时从代码仓库获取资源



又可以分为：

- 用户会碰到/配置的东西（比如 task 和 copilot 文件）
- 用户不太会碰到的东西（比如模型、模板图片、关卡数据）





参考 syncthing：https://docs.syncthing.net/users/syncthing.html

默认 DIR 为：

The default configuration directory is `$XDG_STATE_HOME/syncthing` or `$HOME/.local/state/syncthing` (Unix-like), `$HOME/Library/Application Support/Syncthing` (Mac) and `%LOCALAPPDATA%\Syncthing` (Windows).

可以通过 cli 参数 `--home` 来设置



而 scoop 的 manifest 中：

https://github.com/ScoopInstaller/Main/blob/master/bucket/syncthing.json

```
"bin": [
    [
        "syncthing.exe",
        "syncthing",
        "-home \"$dir\\config\""
    ]
],
```

设置了 alias `syncthing` 为 `syncthing -home "$dir/config"`