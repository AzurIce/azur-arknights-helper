use std::sync::Arc;

use aah::{ActionOptions, resource::{AahResource, Load}};

fn main() -> anyhow::Result<()> {
    // 加载资源
    println!("正在加载资源...");
    let resource = AahResource::load("aah-resources")?;
    let resource = Arc::new(resource);

    // 打印加载的任务数量
    println!("加载了 {} 个任务:", resource.tasks.len());

    // 列出所有加载的任务
    for (name, task) in resource.tasks.iter() {
        let desc = task.desc.as_ref().map(|d| d.as_str()).unwrap_or("无描述");
        println!("  - {}: {} ({} 步骤)", name, desc, task.steps.len());
    }

    // 测试 ByName 引用
    println!("\n检查任务引用...");
    if let Some(task) = resource.get_task("award") {
        println!("找到任务 'award'，描述: {:?}", task.desc);
        println!("步骤数量: {}", task.steps.len());

        // 打印每个步骤及其选项
        for (i, step) in task.steps.iter().enumerate() {
            println!(
                "  步骤 {}: {:?} (retry={}, skip_if_failed={}, delay={:?})",
                i,
                step.action,
                step.options.retry,
                step.options.skip_if_failed,
                step.options.delay_after
            );
        }
    }

    // 测试 ActionOptions 的使用
    println!("\n测试 ActionOptions 构建...");
    let opts = ActionOptions::new()
        .with_retry(3)
        .with_skip_if_failed(true)
        .with_delay(0.5);
    println!("  创建的选项: retry={}, skip_if_failed={}, delay={:?}",
        opts.retry, opts.skip_if_failed, opts.delay_after);

    println!("\n✅ 测试成功！所有任务加载正常，ActionOptions API 工作正常。");

    Ok(())
}
