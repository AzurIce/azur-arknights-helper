mod sub;

use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
    thread,
};

use aah_core::{task::TaskEvt, AAH};
use aah_resource::Resource;
use iced::{
    futures::SinkExt,
    widget::{button, column, container, row, text},
    Alignment, Element, Subscription, Task,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Debug, Clone, Eq, PartialEq)]
enum Tab {
    Main,
    Tasks,
}

struct App {
    tab: Tab,
    log: Vec<String>,
    initializing_resource: bool,
    resource: Option<Arc<Resource>>,
    aah: Option<Arc<Mutex<AAH>>>,
    connecting: bool,

    task_evt_listener_tx: Option<iced::futures::channel::mpsc::UnboundedSender<sub::Input>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            tab: Tab::Main,
            log: vec!["Initializing Resource...".to_string()],
            initializing_resource: true,
            resource: None,
            aah: None,
            connecting: false,
            task_evt_listener_tx: None,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Empty,
    InitResourceRes(Result<Arc<Resource>, String>),
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
        info!("app update message: {:?}", message);
        match message {
            Message::InitResourceRes(res) => match res {
                Ok(resource) => {
                    self.resource = Some(resource);
                    self.initializing_resource = false;
                    self.log.push("Resource Initialized".to_string());
                }
                Err(err) => {
                    self.log
                        .push(format!("Failed to initialize resource: {}", err));
                }
            },
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
            Message::TaskEvt(evt) => {
                self.log.push(format!("task_evt: {:?}", evt));
            }

            Message::RunTask(task_name) => {
                if let Some(aah) = self.aah.clone() {
                    thread::spawn(move || {
                        aah.lock().unwrap().run_task(task_name).unwrap();
                    });
                }
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> iced::Element<Message> {
        let logs = column(self.log.iter().map(|log| text(log).into()));

        let tabs = row![if self.tab == Tab::Main {
            button("Main")
        } else {
            button("Main").on_press(Message::SetTab(Tab::Main))
        }];

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
                let mut view = row![];
                // let tasks = match &self.aah {
                //     Some(aah) => {
                //         let tasks = aah.lock().unwrap().get_tasks();
                //         column![
                //             tasks.iter().map(|task| text(task).into())
                //         ]
                //     }
                //     None => column![text!("No connection")],
                // }
                container(view)
            }
        };
        column![tabs, main, logs].into()
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
