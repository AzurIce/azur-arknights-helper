mod sub;
mod widgets;

use std::{
    fmt::{self, Debug, Display},
    sync::{Arc, Mutex},
    thread,
};

use aah_core::{task::TaskEvt, AAH, resource::Resource};
use iced::{
    color,
    futures::SinkExt,
    widget::{
        button, column, container, horizontal_rule, horizontal_space, image::Handle, row, text,
        text_editor, toggler,
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
                Element::from(button(text(tab_str)))
            } else {
                Element::from(button(text(tab_str)).on_press(CloneMessage::SetTab(tab.clone())))
            }
            .map(Message::CloneMessage)
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
    log_content: text_editor::Content,

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
            log_content: text_editor::Content::new(),
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
enum CloneMessage {
    InitResource,
    RunTask(String),
    Connect,
    Disconnect,
    PrevImg,
    NextImg,
    SetTab(Tab),
    LogEdit(text_editor::Action),
}

impl From<CloneMessage> for Message {
    fn from(msg: CloneMessage) -> Self {
        match msg {
            CloneMessage::InitResource => Message::InitResource,
            CloneMessage::RunTask(task_name) => Message::RunTask(task_name),
            CloneMessage::Connect => Message::Connect,
            CloneMessage::Disconnect => Message::Disconnect,
            CloneMessage::PrevImg => Message::PrevImg,
            CloneMessage::NextImg => Message::NextImg,
            CloneMessage::SetTab(tab) => Message::SetTab(tab),
            CloneMessage::LogEdit(action) => Message::LogEdit(action),
        }
    }
}

#[derive(Debug)]
enum Message {
    CloneMessage(CloneMessage),
    LogEdit(text_editor::Action),

    PrevImg,
    NextImg,
    Empty,
    ToggleDebug(bool),
    InitResource,
    InitResourceRes(Result<Arc<Resource>, String>),
    CheckAndUpdateResource,
    Connect,
    ConnectRes(Result<AAH, String>),
    Disconnect,

    RunTask(String),

    SetTab(Tab),

    /// for task_evt_listener
    TaskEvt(TaskEvt),
    TaskEvtListenerReady(iced::futures::channel::mpsc::UnboundedSender<sub::Input>),
    TaskEvtListenerListening,
}

impl App {
    fn prev_img(&mut self) {
        if self.img_idx > 0 {
            self.img_idx -= 1;
        }
    }

    fn next_img(&mut self) {
        if self.img_idx < self.annotated_imgs.len() - 1 {
            self.img_idx += 1;
        }
    }

    fn toggle_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    fn log(&mut self, s: impl AsRef<str>) {
        let s = s.as_ref().to_string();
        self.log_content
            .perform(text_editor::Action::Move(text_editor::Motion::DocumentEnd));
        self.log_content
            .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                Arc::new(format!("{}\n", s)),
            )));
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        let message = match message {
            Message::CloneMessage(msg) => msg.into(),
            _ => message,
        };
        // info!("app update message: {:?}", message);
        match message {
            Message::ToggleDebug(debug) => self.toggle_debug(debug),
            Message::PrevImg => self.prev_img(),
            Message::NextImg => self.next_img(),
            Message::LogEdit(action) => {
                if !action.is_edit() {
                    self.log_content.perform(action);
                }
            }
            // Message::CheckAndUpdateResource => {
            //     self.checking_resource_update = true;
            //     Task::perform(async {

            //     }, f)
            // }
            Message::InitResource => {
                self.initializing_resource = true;
                self.log("Initializing Resource...");
                return Task::perform(
                    async {
                        Resource::try_init(".aah/resources")
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
                        self.log("Resource Initialized");
                    }
                    Err(err) => {
                        self.log(format!("Failed to initialize resource: {}", err));
                    }
                }
            }
            Message::Connect => {
                self.connecting = true;
                let resource = self.resource.clone().unwrap();
                return Task::perform(
                    async move {
                        AAH::connect("127.0.0.1:16384", resource).map_err(|err| {
                            format!(
                                "Failed to connect: {}, Caused by: {}",
                                err,
                                err.root_cause()
                            )
                        })
                    },
                    Message::ConnectRes,
                );
            }
            Message::ConnectRes(res) => {
                match res {
                    Ok(aah) => {
                        self.aah = Some(Arc::new(Mutex::new(aah)));
                        self.log("connected to 127.0.0.1:16384");
                    }
                    Err(err) => self.log(err),
                }
                self.connecting = false;
            }
            Message::Disconnect => {
                self.aah = None;
                self.log("disconnected");
            }
            Message::SetTab(tab) => {
                self.tab = tab;
            }
            // task_evt_listener stuff
            Message::TaskEvtListenerReady(tx) => {
                self.log("task_evt_listener ready, starting listener");
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
                self.log("task_evt_listener listening");
            }
            Message::TaskEvt(evt) => match evt {
                TaskEvt::AnnotatedImg(img) => {
                    let handle = Handle::from_rgba(img.width(), img.height(), img.into_bytes());
                    self.annotated_imgs.push(handle);
                    self.img_idx = self.annotated_imgs.len() - 1;
                }
                TaskEvt::ExecStat { step, cur, total } => {
                    self.log(format!(
                        "executing {}: {}/{} ({:?})",
                        self.executing_task.as_ref().unwrap_or(&"".to_string()),
                        cur,
                        total,
                        step
                    ));
                }
                TaskEvt::Log(s) => {
                    self.log(s);
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
        let main = match self.tab {
            Tab::Main => {
                let tasks = match &self.aah {
                    Some(aah) => {
                        column![
                            Element::from(
                                button("start_up")
                                    .on_press(CloneMessage::RunTask("start_up".to_string()))
                            )
                            .map(Message::CloneMessage),
                            Element::from(
                                button("award")
                                    .on_press(CloneMessage::RunTask("award".to_string()))
                            )
                            .map(Message::CloneMessage),
                        ]
                    }
                    None => column![text!("No connection")],
                }
                .spacing(2);

                let view = column![tasks].align_x(Alignment::Center).spacing(4);
                container(view)
            }
            Tab::Tasks => {
                if let Some(resource) = &self.resource {
                    let tasks = column(resource.get_tasks().into_iter().map(|task_name| {
                        row![
                            text(task_name.clone()),
                            horizontal_space(),
                            if self.aah.is_none() {
                                Element::from(button("No Connection"))
                            } else if self.executing_task.as_ref() == Some(&task_name) {
                                Element::from(button("Running..."))
                            } else {
                                Element::from(
                                    button("Run")
                                        .on_press(CloneMessage::RunTask(task_name.clone())),
                                )
                            }
                            .map(Message::CloneMessage)
                        ]
                        .align_y(Alignment::Center)
                        .spacing(2)
                        .into()
                    }))
                    .spacing(2);

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
                                Element::from(button("Prev").on_press(CloneMessage::PrevImg))
                            } else {
                                Element::from(button("Prev"))
                            }
                            .map(Message::CloneMessage),
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
                                Element::from(button("Next").on_press(CloneMessage::NextImg))
                            } else {
                                Element::from(button("Next"))
                            }
                            .map(Message::CloneMessage)
                        ]
                        .spacing(4)
                        .align_y(Alignment::Center)
                    ]
                    .spacing(2)
                    .align_x(Alignment::Center);

                    container(
                        row![
                            tasks.width(Length::FillPortion(1)),
                            annotated_img_viewer.width(Length::FillPortion(3))
                        ]
                        .padding(2)
                        .spacing(2),
                    )
                } else {
                    container(text("No Resource"))
                }
            }
        }
        .height(Length::Fill);

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
                    Element::from(button("Initializing Resource..."))
                } else {
                    Element::from(
                        button("Initialize Resource").on_press(CloneMessage::InitResource),
                    )
                }
                .map(Message::CloneMessage)
            ]
        };

        let mut top_bar = row![].align_y(Alignment::Center).spacing(2).padding(2);
        if self.resource.is_some() {
            top_bar = top_bar.push(
                if self.aah.is_none() {
                    if self.connecting {
                        Element::from(button("Connecting..."))
                    } else {
                        Element::from(button("Connect").on_press(CloneMessage::Connect))
                    }
                } else {
                    Element::from(button("Disconnect").on_press(CloneMessage::Disconnect))
                }
                .map(Message::CloneMessage),
            );
        }

        top_bar = top_bar.push(resource_status);
        top_bar = top_bar.push(horizontal_space());
        top_bar = top_bar.push(toggler(self.debug).on_toggle(Message::ToggleDebug));

        // let logs = column(self.log.iter().map(|log| text(log).into()));

        let view: Element<Message> = column![
            top_bar,
            horizontal_rule(1),
            Tab::tab_bar(&self.tab),
            main,
            Element::from(
                text_editor(&self.log_content)
                    .on_action(CloneMessage::LogEdit)
                    .height(Length::Fixed(200.0))
            )
            .map(Message::CloneMessage)
        ]
        .padding(2)
        .into();
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
        .run_with(|| (App::default(), Task::done(Message::InitResource)))
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
