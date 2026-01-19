#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, mpsc};
use std::time::Duration;

use aah::{AahCore, resource::{AahResource, Load}};
use eframe::egui;
use tokio::runtime::Runtime;

enum GuiMessage {
    Log(String),
    Connected(Arc<AahCore>),
    ConnectionError(String),
    TaskStarted(String),
    TaskFinished(String, Result<(), String>),
    ResourceLoaded(Arc<AahResource>),
    ResourceError(String),
}

struct AahGui {
    serial: String,
    resource_path: String,
    selected_task: Option<String>,
    selected_copilot: Option<String>,
    
    runtime: Runtime,
    rx: mpsc::Receiver<GuiMessage>,
    tx: mpsc::Sender<GuiMessage>,
    
    aah: Option<Arc<AahCore>>,
    resource: Option<Arc<AahResource>>,
    logs: Vec<String>,
    is_connecting: bool,
    is_running_task: bool,
}

impl AahGui {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = mpsc::channel();
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");

        let app = Self {
            serial: "127.0.0.1:16384".to_owned(),
            resource_path: "aah-resources".to_owned(),
            selected_task: None,
            selected_copilot: None,
            runtime,
            rx,
            tx: tx.clone(),
            aah: None,
            resource: None,
            logs: vec![],
            is_connecting: false,
            is_running_task: false,
        };
        
        app.load_resources();
        
        app
    }

    fn load_resources(&self) {
        let tx = self.tx.clone();
        let path = self.resource_path.clone();
        self.runtime.spawn(async move {
            match AahResource::load(&path) {
                Ok(res) => {
                    tx.send(GuiMessage::Log(format!("Loaded resources from {}", path))).ok();
                    tx.send(GuiMessage::ResourceLoaded(Arc::new(res))).ok();
                }
                Err(e) => {
                    tx.send(GuiMessage::ResourceError(e.to_string())).ok();
                }
            }
        });
    }

    fn connect(&mut self) {
        if self.resource.is_none() {
            self.log("Cannot connect: Resources not loaded");
            return;
        }
        
        self.is_connecting = true;
        let serial = self.serial.clone();
        let resource = self.resource.clone().unwrap();
        let tx = self.tx.clone();
        
        self.runtime.spawn(async move {
            tx.send(GuiMessage::Log(format!("Connecting to {}...", serial))).ok();
            match AahCore::connect(&serial, resource) {
                Ok(core) => {
                    tx.send(GuiMessage::Connected(Arc::new(core))).ok();
                }
                Err(e) => {
                    tx.send(GuiMessage::ConnectionError(e.to_string())).ok();
                }
            }
        });
    }

    fn run_task(&mut self, task_name: String) {
        if let Some(core) = &self.aah {
            self.is_running_task = true;
            let core = core.clone();
            let tx = self.tx.clone();
            let name = task_name.clone();
            
            self.runtime.spawn(async move {
                tx.send(GuiMessage::TaskStarted(name.clone())).ok();
                match core.run_task(&name) {
                    Ok(_) => tx.send(GuiMessage::TaskFinished(name, Ok(()))).ok(),
                    Err(e) => tx.send(GuiMessage::TaskFinished(name, Err(e.to_string()))).ok(),
                }
            });
        }
    }

    fn run_copilot(&mut self, copilot_name: String) {
        if let Some(core) = &self.aah {
            self.is_running_task = true;
            let core = core.clone();
            let tx = self.tx.clone();
            let name = copilot_name.clone();
            
            self.runtime.spawn(async move {
                tx.send(GuiMessage::TaskStarted(name.clone())).ok();
                match core.run_copilot(&name) {
                    Ok(_) => tx.send(GuiMessage::TaskFinished(name, Ok(()))).ok(),
                    Err(e) => tx.send(GuiMessage::TaskFinished(name, Err(e.to_string()))).ok(),
                }
            });
        }
    }

    fn log(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        println!("{}", msg);
        self.logs.push(format!("[{}] {}", chrono::Local::now().format("%H:%M:%S"), msg));
    }
}

impl eframe::App for AahGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                GuiMessage::Log(msg) => self.log(msg),
                GuiMessage::Connected(core) => {
                    self.is_connecting = false;
                    self.aah = Some(core);
                    self.log("Device connected successfully.");
                },
                GuiMessage::ConnectionError(err) => {
                    self.is_connecting = false;
                    self.log(format!("Connection failed: {}", err));
                },
                GuiMessage::ResourceLoaded(res) => {
                    self.resource = Some(res);
                    self.log("Resources loaded.");
                },
                GuiMessage::ResourceError(err) => {
                    self.log(format!("Failed to load resources: {}", err));
                },
                GuiMessage::TaskStarted(name) => {
                    self.log(format!("Started task: {}", name));
                },
                GuiMessage::TaskFinished(name, result) => {
                    self.is_running_task = false;
                    match result {
                        Ok(_) => self.log(format!("Task '{}' completed successfully.", name)),
                        Err(e) => self.log(format!("Task '{}' failed: {}", name, e)),
                    }
                }
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Serial:");
                ui.text_edit_singleline(&mut self.serial);
                
                if ui.add_enabled(!self.is_connecting && self.aah.is_none(), egui::Button::new("Connect")).clicked() {
                    self.connect();
                }
                
                if self.is_connecting {
                    ui.spinner();
                }
                
                if self.aah.is_some() {
                    ui.label("✅ Connected");
                }
                
                ui.separator();
                
                ui.label("Res Path:");
                ui.text_edit_singleline(&mut self.resource_path);
                if ui.button("Reload").clicked() {
                    self.load_resources();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |columns| {
                columns[0].vertical(|ui| {
                    ui.heading("Tasks / Copilots");
                    
                    if let Some(res) = &self.resource {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.label("Tasks:");
                            for task_name in res.inner.tasks.keys() {
                                if ui.selectable_label(self.selected_task.as_ref() == Some(task_name), task_name).clicked() {
                                    self.selected_task = Some(task_name.clone());
                                    self.selected_copilot = None;
                                }
                            }
                            
                            ui.separator();
                            ui.label("Copilots:");
                            for copilot_name in res.copilot_config.keys() {
                                if ui.selectable_label(self.selected_copilot.as_ref() == Some(copilot_name), copilot_name).clicked() {
                                    self.selected_copilot = Some(copilot_name.clone());
                                    self.selected_task = None;
                                }
                            }
                        });
                    } else {
                        ui.label("No resources loaded.");
                    }
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        if let Some(task) = &self.selected_task {
                            if ui.add_enabled(!self.is_running_task && self.aah.is_some(), egui::Button::new("Run Task")).clicked() {
                                self.run_task(task.clone());
                            }
                        }
                        if let Some(copilot) = &self.selected_copilot {
                            if ui.add_enabled(!self.is_running_task && self.aah.is_some(), egui::Button::new("Run Copilot")).clicked() {
                                self.run_copilot(copilot.clone());
                            }
                        }
                    });
                });

                columns[1].vertical(|ui| {
                    ui.heading("Logs");
                    egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                        for log in &self.logs {
                            ui.label(log);
                        }
                    });
                });
            });
        });
        
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "AAH GUI",
        options,
        Box::new(|cc| Ok(Box::new(AahGui::new(cc)))),
    )
}