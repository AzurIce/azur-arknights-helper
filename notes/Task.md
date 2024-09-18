```toml
name = "mytask"
desc = "some description about it"

[[steps]]
skip_if_failed = true
[steps.action.XXX]
xxx
```



每个 Task 分为多个 Step，在每一个 Step 会执行一个 Action。
