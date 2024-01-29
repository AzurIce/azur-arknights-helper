#![feature(associated_type_defaults)]
#![feature(path_file_prefix)]

use std::{error::Error, fs};

use config::{navigate::NavigateConfig, task::TaskConfig};
use controller::{AdbInputController, Controller};
use ocrs::{OcrEngine, OcrEngineParams};
use rten::Model;

use crate::task::Task;

pub mod adb;
pub mod config;
pub mod controller;
pub mod task;
pub mod vision;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

fn try_init_ocr_engine() -> Result<OcrEngine, Box<dyn Error>> {
    println!("Initializing ocr engine...");
    if fs::File::open("text-detection.rten").is_err() {
        let client = reqwest::blocking::get("https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten")?;
        fs::write("text-detection.rten", client.bytes()?)?;
    }
    if fs::File::open("text-recognition.rten").is_err() {
        let client = reqwest::blocking::get("https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.rten")?;
        fs::write("text-recognition.rten", client.bytes()?)?;
    }

    let detection_model_data = fs::read("text-detection.rten")?;
    let rec_model_data = fs::read("text-recognition.rten")?;

    let detection_model = Model::load(&detection_model_data)?;
    let recognition_model = Model::load(&rec_model_data)?;

    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })?;
    Ok(engine)
}

pub struct AAH {
    pub controller: Box<dyn Controller>,
    pub task_config: TaskConfig,
    pub navigate_config: NavigateConfig,
    pub ocr_engine: Option<OcrEngine>,
}

impl AAH {
    pub fn connect<S: AsRef<str>>(serial: S) -> Result<Self, Box<dyn Error>> {
        let task_config = TaskConfig::load("./resources")?;
        let navigate_config = NavigateConfig::load("./resources")?;
        let controller = Box::new(AdbInputController::connect(serial)?);
        Ok(Self {
            controller,
            task_config,
            navigate_config,
            ocr_engine: Some(try_init_ocr_engine()?),
        })
    }

    pub fn run_task<S: AsRef<str>>(&self, name: S) -> Result<(), String> {
        let name = name.as_ref().to_string();

        let task = self
            .task_config
            .0
            .get(&name)
            .ok_or("failed to get task")?
            .clone();
        println!("executing {:?}", task);

        task.run(self)?;

        Ok(())
    }
}
