use std::{
    collections::{HashMap, HashSet},
    thread,
    time::Duration,
};

use aah_cv::template_matching::{match_template, MatchTemplateMethod};
use aah_resource::level::get_level;
use color_print::{cformat, cprintln};
use imageproc::template_matching::find_extremes;

use crate::{
    config::copilot::{Copilot, CopilotAction, CopilotStepTime},
    task::{
        action::click_match::ClickMatch,
        battle::{Deploy, Retreat, UseSkill},
        match_task::MatchTask,
        Runnable, TaskEvt,
    },
    utils::resource::get_template,
    vision::analyzer::{
        battle::{BattleAnalyzer, BattleAnalyzerOutput, BattleState},
        Analyzer,
    },
};

#[derive(Default)]
struct CopilotState {
    /// name to tile_pos
    deployed_operators: HashMap<String, (u32, u32)>,
    auto_skill_operators: HashSet<String>,
}

impl CopilotState {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Runnable for Copilot {
    type Err = String;
    fn run(&self, aah: &crate::AAH) -> Result<Self::Res, Self::Err> {
        let log_tag = cformat!("<strong>[CopilotTask {}]: </strong>", self.level_code);
        let copilot_task = aah.copilot_config.get_task(&self.level_code)?;
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
        let start_pre = ClickMatch::new(MatchTask::Template("level_start-pre.png".to_string()));
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
        let start_pre = ClickMatch::new(MatchTask::Template("formation_start.png".to_string()));
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
        let mut battle_analyzer_output: BattleAnalyzerOutput;
        let mut deployed_operators = HashMap::<String, (u32, u32)>::new();
        let mut auto_skill_operators = HashSet::<String>::new();

        let auto_skilling =
            |auto_skill_operators: &HashSet<String>,
             deployed_operators: &HashMap<String, (u32, u32)>| {
                aah.emit_task_evt(TaskEvt::Log("[INFO]: 正在检测技能".to_string()));
                cprintln!("{log_tag}checking auto_skill...");
                for oper in auto_skill_operators.iter() {
                    if let Some(position) = deployed_operators.get(oper).cloned() {
                        if let Ok(screen) = aah.screen_cache_or_cap() {
                            let (tile_screen_x, tile_screen_y) =
                                level.calc_tile_screen_pos(position.0, position.1, false);
                            let skill_cropped = screen.crop_imm(
                                (tile_screen_x as u32).saturating_add_signed(-32),
                                (tile_screen_y as u32).saturating_add_signed(-187),
                                64,
                                64,
                            );
                            // skill_cropped.save("./output.png").unwrap();
                            let res = match_template(
                                &skill_cropped.to_luma32f(),
                                &skill_ready_template,
                                MatchTemplateMethod::CrossCorrelationNormed,
                                false,
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
                                if UseSkill::new(&level, &position).run(aah).is_ok() {
                                    aah.emit_task_evt(TaskEvt::Log(format!(
                                        "[INFO]: {oper} 技能已使用"
                                    )));
                                    cprintln!("{log_tag}auto_skill[{oper}]: skill clicked");
                                    thread::sleep(Duration::from_secs_f32(0.2))
                                }
                            }
                        }
                    }
                }
            };

        let mut iter = copilot_task.steps.iter().enumerate();
        let mut cur = iter.next();
        while battle_analyzer.battle_state != BattleState::Completed {
            // Execute step
            if let Some((idx, step)) = cur {
                aah.emit_task_evt(TaskEvt::Log(format!(
                    "[INFO]: 执行命令 [{}/{}]: {:?}",
                    idx,
                    iter.len(),
                    step
                )));
                cprintln!(
                    "{log_tag}executing command[{}/{}]: {:?}",
                    idx,
                    iter.len(),
                    step
                );
                aah.emit_task_evt(TaskEvt::Log(format!("[INFO]: 等待 {:?}...", step.time)));
                cprintln!("{log_tag}waiting for time {:?}...", step.time);
                match step.time {
                    CopilotStepTime::DeltaSec(delta) => {
                        thread::sleep(Duration::from_secs_f32(delta));
                    }
                    CopilotStepTime::Asap => (),
                }
                aah.emit_task_evt(TaskEvt::Log(format!("[INFO]: 等待完成")));
                cprintln!("{log_tag}is time!");
                battle_analyzer_output = battle_analyzer.analyze(aah)?;
                // auto skilling
                auto_skilling(&auto_skill_operators, &deployed_operators);

                let success = match &step.action {
                    CopilotAction::Deploy {
                        operator,
                        position,
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
                        let success = battle_analyzer_output
                            .deploy_cards
                            .iter()
                            .find(|card| {
                                card.oper_name.as_str()
                                    == copilot_task.operators.get(operator).unwrap()
                                    && card.available == true
                            })
                            .and_then(|deploy_card| {
                                cprintln!(
                                    "deploying operator {} to {:?}[{}]...",
                                    deploy_card.oper_name,
                                    position,
                                    direction
                                );
                                Deploy::new(&level, &deploy_card.rect, position, direction)
                                    .run(aah)
                                    .ok()
                            })
                            .is_some();
                        if success {
                            deployed_operators.insert(operator.to_string(), *position);
                        }
                        success
                    }
                    CopilotAction::Retreat { operator, .. } => {
                        cprintln!("{log_tag} retreating {operator}...");
                        aah.emit_task_evt(TaskEvt::Log(format!(
                            "[INFO]: 正在撤退干员 {operator}..."
                        )));
                        let position = deployed_operators.get(operator).unwrap();
                        let success = Retreat::new(&level, &position).run(aah).is_ok();
                        if success {
                            deployed_operators.remove(operator);
                            aah.emit_task_evt(TaskEvt::Log(format!("[INFO]: {operator} 已撤退")));
                        }
                        success
                    }
                    CopilotAction::AutoSkill { operator, .. } => {
                        cprintln!("{log_tag}enable auto_skill for {operator}...");
                        auto_skill_operators.insert(operator.to_string());
                        true
                    }
                    CopilotAction::StopAutoSkill { operator, .. } => {
                        cprintln!("{log_tag}disable auto_skill for {operator}...");
                        auto_skill_operators.remove(operator);
                        true
                    }
                };
                if success {
                    cprintln!("{log_tag}<green>command success!</green>");
                    cur = iter.next();
                }
            }

            battle_analyzer.analyze(aah)?;
            // auto skilling
            auto_skilling(&auto_skill_operators, &deployed_operators);
        }

        Ok(())
    }
}
