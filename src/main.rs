mod sub;
mod widgets;

use core::task;
use std::{
    fmt::{self, Debug, Display},
    sync::{Arc, Mutex},
    thread,
};

use ::image::DynamicImage;
use aah_core::{task::TaskEvt, AAH};
use aah_resource::Resource;
use iced::{
    color,
    futures::SinkExt,
    widget::{
        button, column, container, horizontal_space,
        image::{self, Handle},
        row, text, toggler,
    },
    Alignment, Element, Length, Subscription, Task,
};
use tracing::error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Debug, Clone, Eq, PartialEq)]
enum Tab {
    Main,
    Tasks,
}

impl Tab {
    const ALL: [Tab; 2] = [Tab::Main, Tab::Tasks];

    fn tab_bar(cur_tab: &Tab) -> iced::Element<Message> {
        row(Tab::ALL.iter().map(|tab| {
            let tab_str = tab.to_string();

            if cur_tab == tab {
                button(text(tab_str))
            } else {
                button(text(tab_str)).on_press(Message::SetTab(tab.clone()))
            }
            .into()
        }))
        .spacing(2)
        .padding(2)
        .width(Length::Fill)
        .into()
    }
}

impl Display for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

struct App {
    debug: bool,
    tab: Tab,
    log: Vec<String>,
    initializing_resource: bool,
    resource: Option<Arc<Resource>>,
    aah: Option<Arc<Mutex<AAH>>>,
    connecting: bool,

    annotated_imgs: Vec<Handle>,
    img_idx: usize,
    executing_task: Option<String>,
    task_evt_listener_tx: Option<iced::futures::channel::mpsc::UnboundedSender<sub::Input>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            debug: false,
            tab: Tab::Main,
            log: vec!["Initializing Resource...".to_string()],
            initializing_resource: true,
            resource: None,
            aah: None,
            connecting: false,
            annotated_imgs: vec![],
            img_idx: 0,
            executing_task: None,
            task_evt_listener_tx: None,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    PrevImg,
    NextImg,

    Empty,
    ToggleDebug(bool),
    InitResource,
    InitResourceRes(Result<Arc<Resource>, String>),
    CheckAndUpdateResource,
    Connect,
    Disconnect,

    RunTask(String),

    SetTab(Tab),

    /// for task_evt_listener
    TaskEvt(TaskEvt),
    TaskEvtListenerReady(iced::futures::channel::mpsc::UnboundedSender<sub::Input>),
    TaskEvtListenerListening,
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        // info!("app update message: {:?}", message);
        match message {
            Message::ToggleDebug(debug) => {
                self.debug = debug;
            }
            Message::PrevImg => {
                if self.img_idx > 0 {
                    self.img_idx -= 1;
                }
            }
            Message::NextImg => {
                if self.img_idx < self.annotated_imgs.len() - 1 {
                    self.img_idx += 1;
                }
            }
            // Message::CheckAndUpdateResource => {
            //     self.checking_resource_update = true;
            //     Task::perform(async {

            //     }, f)
            // }
            Message::InitResource => {
                self.initializing_resource = true;
                Task::perform(
                    async {
                        Resource::try_init(".aah")
                            .await
                            .map(|res| Arc::new(res))
                            .map_err(|err| {
                                format!(
                                    "Failed to initialize resource: {}, Caused by: {}",
                                    err,
                                    err.root_cause()
                                )
                            })
                    },
                    Message::InitResourceRes,
                );
            }
            Message::InitResourceRes(res) => {
                self.initializing_resource = false;
                match res {
                    Ok(resource) => {
                        self.resource = Some(resource);
                        self.log.push("Resource Initialized".to_string());
                    }
                    Err(err) => {
                        self.log
                            .push(format!("Failed to initialize resource: {}", err));
                    }
                }
            }
            Message::Connect => {
                self.connecting = true;
                match AAH::connect("127.0.0.1:16384", self.resource.clone().unwrap()) {
                    Ok(aah) => {
                        self.aah = Some(Arc::new(Mutex::new(aah)));
                        self.log.push("connected to 127.0.0.1:16384".to_string());
                    }
                    Err(err) => self.log.push(format!(
                        "Failed to connect: {}, Caused by: {}",
                        err,
                        err.root_cause()
                    )),
                }
                self.connecting = false;
            }
            Message::Disconnect => {
                self.aah = None;
                self.log.push("disconnected".to_string());
            }
            Message::SetTab(tab) => {
                self.tab = tab;
            }
            // task_evt_listener stuff
            Message::TaskEvtListenerReady(tx) => {
                self.log
                    .push("task_evt_listener ready, starting listener".to_string());
                self.task_evt_listener_tx = Some(tx);

                if let Some(aah) = &self.aah {
                    let mut tx = self.task_evt_listener_tx.as_ref().unwrap().clone();
                    let rx = aah.lock().unwrap().task_evt_rx.clone();
                    return Task::perform(
                        async move { tx.send(sub::Input::StartListenToTaskEvt(rx)).await },
                        |_| Message::Empty,
                    );
                }
            }
            Message::TaskEvtListenerListening => {
                self.log.push("task_evt_listener listening".to_string());
            }
            Message::TaskEvt(evt) => match evt {
                TaskEvt::AnnotatedImg(img) => {
                    let handle = Handle::from_rgba(img.width(), img.height(), img.into_bytes());
                    self.annotated_imgs.push(handle);
                    self.img_idx = self.annotated_imgs.len() - 1;
                }
                TaskEvt::ExecStat { step, cur, total } => {
                    self.log.push(format!(
                        "executing {}: {}/{} ({:?})",
                        self.executing_task.as_ref().unwrap_or(&"".to_string()),
                        cur,
                        total,
                        step
                    ));
                }
                TaskEvt::Log(s) => {
                    self.log.push(s);
                }
                _ => {}
            },

            Message::RunTask(task_name) => {
                if let Some(aah) = self.aah.clone() {
                    self.annotated_imgs.clear();
                    self.img_idx = 0;
                    thread::spawn(move || {
                        let res = aah.lock().unwrap().run_task(&task_name);
                        if let Err(err) = res {
                            error!("Failed to run task {}: {}", task_name, err);
                            // error!("Failed to run task {}: {}, Caused by: {}", task_name, err, err.root_cause());
                        }
                    });
                }
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> iced::Element<Message> {
        let logs = column(self.log.iter().map(|log| text(log).into()));

        let resource_status = if let Some(resource) = &self.resource {
            row![
                text("Resource Initialized: "),
                text(resource.manifest.last_updated.to_string()),
            ]
            // if self.checking_resource_update {
            //     resource_status = resource_status.push(button("Checking Update..."));
            // } else {
            //     resource_status = resource_status.push(button("Check Update").on_press(Message::CheckAndUpdateResource));
            // }
        } else {
            row![
                text("Resource Not Initialized: "),
                if self.initializing_resource {
                    button("Initializing Resource...")
                } else {
                    button("Initialize Resource").on_press(Message::InitResource)
                }
            ]
        };

        let main = match self.tab {
            Tab::Main => {
                let tasks = match &self.aah {
                    Some(aah) => {
                        column![
                            button("start_up").on_press(Message::RunTask("start_up".to_string())),
                            button("award").on_press(Message::RunTask("award".to_string())),
                        ]
                    }
                    None => column![text!("No connection")],
                }
                .spacing(2);

                let mut view = column![tasks].align_x(Alignment::Center).spacing(4);
                if self.resource.is_some() {
                    if self.aah.is_none() {
                        if self.connecting {
                            view = view.push(button("Connecting..."));
                        } else {
                            view = view.push(button("Connect").on_press(Message::Connect));
                        }
                    } else {
                        view = view.push(button("Disconnect").on_press(Message::Disconnect));
                    }
                }
                container(view)
            }
            Tab::Tasks => {
                if let Some(resource) = &self.resource {
                    let tasks = column(resource.get_tasks().into_iter().map(|task_name| {
                        row![
                            text(task_name.clone()),
                            horizontal_space(),
                            if self.aah.is_none() {
                                button("No Connection")
                            } else if self.executing_task.as_ref() == Some(&task_name) {
                                button("Running...")
                            } else {
                                button("Run").on_press(Message::RunTask(task_name.clone()))
                            }
                        ]
                        .spacing(2)
                        .width(Length::Shrink)
                        .into()
                    }));

                    let annotated_img = if let Some(img) = self.annotated_imgs.get(self.img_idx) {
                        container(
                            iced::widget::image(img)
                                .width(Length::Fill)
                                .height(Length::Fill),
                        )
                    } else {
                        container(text("No annotated image"))
                    }
                    .center(Length::Fill);

                    let annotated_img_viewer = column![
                        annotated_img,
                        row![
                            if self.img_idx > 0 {
                                button("Prev").on_press(Message::PrevImg)
                            } else {
                                button("Prev")
                            },
                            horizontal_space(),
                            text(format!(
                                "{} / {}",
                                if self.annotated_imgs.is_empty() {
                                    0
                                } else {
                                    self.img_idx + 1
                                },
                                self.annotated_imgs.len()
                            )),
                            horizontal_space(),
                            if self.img_idx + 1 < self.annotated_imgs.len() {
                                button("Next").on_press(Message::NextImg)
                            } else {
                                button("Next")
                            },
                        ]
                        .spacing(4)
                        .align_y(Alignment::Center)
                    ]
                    .spacing(2)
                    .align_x(Alignment::Center);

                    container(row![tasks, annotated_img_viewer])
                } else {
                    container(text("No Resource"))
                }
            }
        }
        .height(Length::Fill);

        let top_bar = row![
            resource_status,
            horizontal_space(),
            toggler(self.debug).on_toggle(Message::ToggleDebug)
        ]
        .align_y(Alignment::Center);
        let view: Element<Message> = column![top_bar, Tab::tab_bar(&self.tab), main, logs].into();
        if self.debug {
            view.explain(color!(0xf06090))
        } else {
            view
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.aah.is_some() {
            return Subscription::run(sub::task_evt_listener).map(|msg| match msg {
                sub::Event::Ready(tx) => Message::TaskEvtListenerReady(tx),
                sub::Event::ListeningToTaskEvt => Message::TaskEvtListenerListening,
                sub::Event::TaskEvt(evt) => Message::TaskEvt(evt),
            });
        }
        Subscription::none()
    }
}

// fn task(task: String, resource: Arc<Resource>) -> Element<Message> {
//     resource.
// }

fn main() -> iced::Result {
    init_logger();

    iced::application("azur-arknights-helper", App::update, App::view)
        .subscription(App::subscription)
        .run_with(|| {
            (
                App::default(),
                Task::perform(
                    async {
                        tokio::task::spawn(Resource::try_init(".aah"))
                            .await
                            .unwrap()
                            .map(|res| Arc::new(res))
                            .map_err(|err| {
                                format!(
                                    "Failed to initialize resource: {}, Caused by: {}",
                                    err,
                                    err.root_cause()
                                )
                            })
                    },
                    Message::InitResourceRes,
                ),
            )
        })
}

fn init_logger() {
    // let indicatif_layer = IndicatifLayer::new();

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("aah=info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(
            tracing_subscriber::fmt::layer(), // .with_level(false)
                                              // .with_target(false)
                                              // .without_time()
                                              // .with_writer(indicatif_layer.get_stderr_writer()),
        )
        // .with(indicatif_layer)
        .init();
}
