use std::{
    collections::{HashMap, HashSet},
    thread,
    time::Duration,
};

use aah_cv::template_matching::match_template_ccorr_normed;
use aah_resource::level::get_level;
use color_print::{cformat, cprintln};
use imageproc::template_matching::find_extremes;
use serde::{Deserialize, Serialize};

use crate::{
    config::copilot::{BattleCommand, BattleCommandTime, Direction},
    task::{
        builtins::{ActionClick, ActionSwipe},
        wrapper::GenericTaskWrapper,
        TaskEvt,
    },
    vision::{
        analyzer::{
            battle::{BattleAnalyzer, BattleAnalyzerOutput, BattleState},
            Analyzer,
        },
        utils::resource::get_template,
    },
};

use super::{builtins::ActionClickMatch, match_task::MatchTask, Task};

#[derive(Debug, Serialize, Deserialize)]
pub struct CopilotTask(pub String);

impl Task for CopilotTask {
    type Err = String;
    fn run(&self, aah: &crate::AAH) -> Result<Self::Res, Self::Err> {
        let log_tag = cformat!("<strong>[CopilotTask {}]: </strong>", self.0);
        let copilot_task = aah.copilot_config.get_task(&self.0)?;
        let level = get_level(
            copilot_task.level_code.as_str(),
            aah.res_dir.join("levels.json"),
        )
        .unwrap();

        // disable prts
        // TODO: fix it
        // cprintln!("{log_tag}disabling prts...");
        // let disable_prts_task =
        //     ActionClickMatch::new(MatchTask::Template("prts-enabled.png".to_string()), None);
        // match disable_prts_task.run(aah) {
        //     Ok(_) => cprintln!("disabled prts"),
        //     Err(err) => cprintln!("failed to disable prts: {:?}, skipping", err),
        // }

        cprintln!("{log_tag}clicking start-pre...");
        aah.emit_task_evt(TaskEvt::Log("[INFO]: 正在点击 start-pre...".to_string()));
        let start_pre =
            ActionClickMatch::new(MatchTask::Template("level_start-pre.png".to_string()), None);
        match start_pre.run(aah) {
            Ok(_) => {
                aah.emit_task_evt(TaskEvt::Log("[INFO]: 已点击 start-pre".to_string()));
                cprintln!("{log_tag}<g>clicked start pre</g>")
            }
            Err(err) => {
                let err = format!("failed to click start pre: {}", err);
                aah.emit_task_evt(TaskEvt::Log(format!("[ERROR]: {err}")));
                cprintln!("{log_tag}<r>{}</r>", err);
                return Err(err);
            }
        }

        thread::sleep(Duration::from_secs_f32(0.5));
        // TODO: formation

        cprintln!("{log_tag}clicking start...");
        aah.emit_task_evt(TaskEvt::Log("[INFO]: 正在点击 start...".to_string()));
        let start_pre =
            ActionClickMatch::new(MatchTask::Template("formation_start.png".to_string()), None);
        match start_pre.run(aah) {
            Ok(_) => {
                aah.emit_task_evt(TaskEvt::Log("[INFO]: 已点击 start".to_string()));
                cprintln!("{log_tag}<g>clicked start</g>")
            }
            Err(err) => {
                let err = format!("failed to click start: {}", err);
                aah.emit_task_evt(TaskEvt::Log(format!("[ERROR]: {err}")));
                cprintln!("{log_tag}<r>{}</r>", err);
                return Err(err);
            }
        }

        let level_retreat_screen_pos = level.get_retreat_screen_pos();
        let level_skill_screen_pos = level.get_skill_screen_pos();
        let mut battle_analyzer =
            BattleAnalyzer::new(&aah.res_dir, copilot_task.operators.values().collect());
        // wait for battle begins
        aah.emit_task_evt(TaskEvt::Log("[INFO]: 正在等待关卡开始...".to_string()));
        cprintln!("{log_tag}waiting for battle to begin...");
        while battle_analyzer.battle_state == BattleState::Unknown {
            thread::sleep(Duration::from_secs_f32(0.5));
            battle_analyzer.analyze(aah)?;
        }
        // Do battle things
        aah.emit_task_evt(TaskEvt::Log("[INFO]: 关卡开始".to_string()));
        cprintln!("{log_tag}battle begins!");
        let skill_ready_template =
            get_template("battle_skill-ready.png", &aah.res_dir)?.to_luma32f();
        let mut commands = copilot_task.steps.iter().enumerate();
        let mut battle_analyzer_output: BattleAnalyzerOutput;
        let mut deployed_operators = HashMap::<String, (u32, u32)>::new();
        let mut auto_skill_operators = HashSet::<String>::new();

        let auto_skilling =
            |auto_skill_operators: &HashSet<String>,
             deployed_operators: &HashMap<String, (u32, u32)>| {
                aah.emit_task_evt(TaskEvt::Log("[INFO]: 正在检测技能".to_string()));
                cprintln!("{log_tag}checking auto_skill...");
                for oper in auto_skill_operators.iter() {
                    if let Some(tile_pos) = deployed_operators.get(oper).cloned() {
                        if let Ok(screen) = aah.screen_cache_or_cap() {
                            let (tile_screen_x, tile_screen_y) =
                                level.calc_tile_screen_pos(tile_pos.0, tile_pos.1, false);
                            let skill_cropped = screen.crop_imm(
                                (tile_screen_x as u32).saturating_add_signed(-32),
                                (tile_screen_y as u32).saturating_add_signed(-187),
                                64,
                                64,
                            );
                            // skill_cropped.save("./output.png").unwrap();
                            let res = match_template_ccorr_normed(
                                &skill_cropped.to_luma32f(),
                                &skill_ready_template,
                            );
                            let v = find_extremes(&res).max_value;
                            let skill_ready = v > 0.9;
                            aah.emit_task_evt(TaskEvt::Log(format!("[INFO]: {oper} 匹配度：{v}")));
                            cprintln!("{log_tag}{oper}'s skill match is {}", v);
                            // let skill_ready =
                            //     get_skill_ready(&skill_cropped, &aah.res_dir).unwrap() == 1;
                            if skill_ready {
                                aah.emit_task_evt(TaskEvt::Log(format!(
                                    "[INFO]: {oper} 技能就绪，正在使用..."
                                )));
                                cprintln!("{log_tag}{oper}'s skil is ready, clicking...");
                                // 32 187 64x64
                                let click1 =
                                    level.calc_tile_screen_pos(tile_pos.0, tile_pos.1, false);
                                let task1 =
                                    ActionClick::new(click1.0 as u32, click1.1 as u32, None);
                                let task2 = ActionClick::new(
                                    level_skill_screen_pos.0 as u32,
                                    level_skill_screen_pos.1 as u32,
                                    Some(GenericTaskWrapper {
                                        delay: 0.2,
                                        ..Default::default()
                                    }),
                                );
                                if task1.run(aah).and(task2.run(aah)).is_ok() {
                                    aah.emit_task_evt(TaskEvt::Log(format!(
                                        "[INFO]: {oper} 技能已使用"
                                    )));
                                    cprintln!("{log_tag}auto_skill[{oper}]: skill clicked");
                                }
                            }
                        }
                    }
                }
            };

        while battle_analyzer.battle_state != BattleState::Completed {
            if let Some((idx, cmd)) = commands.next() {
                aah.emit_task_evt(TaskEvt::Log(format!(
                    "[INFO]: 执行命令 [{}/{}]: {:?}",
                    idx,
                    commands.len(),
                    cmd
                )));
                cprintln!(
                    "{log_tag}executing command[{}/{}]: {:?}",
                    idx,
                    commands.len(),
                    cmd
                );
                aah.emit_task_evt(TaskEvt::Log(format!("[INFO]: 等待 {:?}...", cmd.time())));
                cprintln!("{log_tag}waiting for time {:?}...", cmd.time());
                match cmd.time() {
                    BattleCommandTime::DeltaSec(delta) => {
                        thread::sleep(Duration::from_secs_f32(delta));
                    }
                    BattleCommandTime::Asap => (),
                }
                aah.emit_task_evt(TaskEvt::Log(format!("[INFO]: 等待完成")));
                cprintln!("{log_tag}is time!");
                let mut success = false;
                while !success {
                    battle_analyzer_output = battle_analyzer.analyze(aah)?;
                    // auto skilling
                    auto_skilling(&auto_skill_operators, &deployed_operators);

                    match cmd {
                        BattleCommand::Deploy {
                            operator,
                            tile,
                            direction,
                            ..
                        } => {
                            aah.emit_task_evt(TaskEvt::Log(format!(
                                "[INFO]: 正在匹配 {operator} 部署卡片..."
                            )));
                            cprintln!(
                                "{log_tag}looking for operator's deploy card {}...",
                                operator
                            );
                            if let Some(card) =
                                battle_analyzer_output.deploy_cards.iter().find(|card| {
                                    card.oper_name.as_str()
                                        == copilot_task.operators.get(operator).unwrap()
                                        && card.available == true
                                })
                            {
                                aah.emit_task_evt(TaskEvt::Log(format!(
                                    "[INFO]: 已找到，正在计算地块屏幕坐标..."
                                )));
                                cprintln!("{log_tag}found! {:?}", card);
                                cprintln!("{log_tag}calculating tile screen pos...");
                                let tile_pos = level.calc_tile_screen_pos(tile.0, tile.1, true);
                                aah.emit_task_evt(TaskEvt::Log(format!(
                                    "[INFO]: 地块屏幕坐标 {tile_pos:?}"
                                )));
                                cprintln!("{log_tag}tile screen pos: {:?}", tile_pos);
                                let task1 = ActionSwipe::new(
                                    (card.rect.x, card.rect.y),
                                    (tile_pos.0 as i32, tile_pos.1 as i32),
                                    0.2,
                                    None,
                                );
                                let swipe_delta = 400;
                                let swipe_end = match direction {
                                    Direction::Up => {
                                        (tile_pos.0 as i32, tile_pos.1 as i32 - swipe_delta)
                                    }
                                    Direction::Right => {
                                        (tile_pos.0 as i32 + swipe_delta, tile_pos.1 as i32)
                                    }
                                    Direction::Down => {
                                        (tile_pos.0 as i32, tile_pos.1 as i32 + swipe_delta)
                                    }
                                    Direction::Left => {
                                        (tile_pos.0 as i32 - swipe_delta, tile_pos.1 as i32)
                                    }
                                };
                                let task2 = ActionSwipe::new(
                                    (tile_pos.0 as u32, tile_pos.1 as u32),
                                    swipe_end,
                                    0.2,
                                    Some(GenericTaskWrapper {
                                        delay: 0.2,
                                        ..Default::default()
                                    }),
                                );
                                if task1.run(aah).and(task2.run(aah)).is_ok() {
                                    deployed_operators.insert(operator.to_string(), *tile);
                                    success = true;
                                }
                            }
                        }
                        BattleCommand::Retreat { operator, .. } => {
                            cprintln!("{log_tag} retreating {operator}...");
                            aah.emit_task_evt(TaskEvt::Log(format!(
                                "[INFO]: 正在撤退干员 {operator}..."
                            )));
                            if let Some(tile_pos) = deployed_operators.get(operator).cloned() {
                                let click1 =
                                    level.calc_tile_screen_pos(tile_pos.0, tile_pos.1, false);
                                let task1 =
                                    ActionClick::new(click1.0 as u32, click1.1 as u32, None);
                                let task2 = ActionClick::new(
                                    level_retreat_screen_pos.0 as u32,
                                    level_retreat_screen_pos.1 as u32,
                                    Some(GenericTaskWrapper {
                                        delay: 0.2,
                                        ..Default::default()
                                    }),
                                );
                                if task1.run(aah).and(task2.run(aah)).is_ok() {
                                    deployed_operators.remove(operator);
                                    aah.emit_task_evt(TaskEvt::Log(format!(
                                        "[INFO]: {operator} 已撤退"
                                    )));
                                    success = true;
                                }
                            }
                        }
                        BattleCommand::AutoSkill { operator, .. } => {
                            cprintln!("{log_tag}enable auto_skill for {operator}...");
                            auto_skill_operators.insert(operator.to_string());
                            success = true;
                        }
                        BattleCommand::StopAutoSkill { operator, .. } => {
                            cprintln!("{log_tag}disable auto_skill for {operator}...");
                            auto_skill_operators.remove(operator);
                            success = true;
                        }
                    }
                }
                cprintln!("{log_tag}command done!");
            }

            battle_analyzer.analyze(aah)?;
            // auto skilling
            auto_skilling(&auto_skill_operators, &deployed_operators);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{task::Task, AAH};

    fn task() -> CopilotTask {
        CopilotTask("1-4".to_string())
    }

    #[test]
    fn foo() {
        let aah = AAH::connect("127.0.0.1:16384", "../../resources", |_| {}).unwrap();
        let task = task();
        task.run(&aah).unwrap();
    }
}
