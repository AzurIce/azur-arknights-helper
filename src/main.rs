use std::{fmt::Debug, sync::Arc};

use aah_core::{task::TaskEvt, AAH};
use aah_resource::Resource;
use iced::{
    widget::{button, column, text},
    Subscription, Task,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

struct App {
    log: Vec<String>,
    initializing_resource: bool,
    resource: Option<Arc<Resource>>,
    aah: Option<AAH>,
    connecting: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            log: vec!["Initializing Resource...".to_string()],
            initializing_resource: true,
            resource: None,
            aah: None,
            connecting: false,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    InitResourceRes(Result<Arc<Resource>, String>),
    Connect,
    Disconnect,
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        info!("message: {:?}", message);
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
                match AAH::connect(
                    "127.0.0.1:16384",
                    &self.resource.as_ref().unwrap().root,
                    |evt| match evt {
                        TaskEvt::Log(s) => {
                            info!("{}", s);
                        }
                        _ => {}
                    },
                ) {
                    Ok(aah) => {
                        self.aah = Some(aah);
                        self.log.push("connected to 127.0.0.1:16384".to_string());
                    }
                    Err(err) => self.log.push(format!("Failed to connect: {}", err)),
                }
            }
            Message::Disconnect => {
                self.aah = None;
                self.log.push("disconnected".to_string());
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> iced::Element<Message> {
        let logs = column(self.log.iter().map(|log| text(log).into()));

        let mut main = column![];
        if self.resource.is_some() {
            if self.aah.is_none() {
                if self.connecting {
                    main = main.push(text("Connecting..."));
                } else {
                    main = main.push(button("Connect").on_press(Message::Connect));
                }
            } else {
                main = main.push(button("Disconnect").on_press(Message::Disconnect));
            }
        }
        column![main, logs].into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

fn main() -> iced::Result {
    init_logger();

    iced::application("azur-arknights-helper", App::update, App::view)
        .subscription(App::subscription)
        .run_with(|| {
            (
                App::default(),
                Task::perform(
                    async {
                        tokio::task::spawn_blocking(|| Resource::try_init(".aah"))
                            .await
                            .unwrap()
                            .map(|res| Arc::new(res))
                            .map_err(|err| format!("Failed to initialize resource: {}", err))
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
