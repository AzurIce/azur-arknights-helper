mod sub;

use aah_core::AAH;
use iced::{
    widget::{column, text, button},
    Subscription, Task,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Default)]
struct App {
    log: Vec<String>,
    aah: Option<AAH>,
    resource_initialized: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Event(sub::Event),
    Btn
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        info!("message: {:?}", message);
        match message {
            Message::Event(evt) => {
                match evt {
                    sub::Event::InitializingResource => {
                        self.log.push("InitializingResource".to_string());
                    }
                    sub::Event::ResourceInitialized => {
                        self.resource_initialized = true;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> iced::Element<Message> {
        column![
            // Log
            button("Init Resource").on_press(Message::Btn),
            column(self.log.iter().map(|log| text(log).into()))
        ]
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        if !self.resource_initialized {
            return Subscription::run(sub::init_resource).map(Message::Event);
        }
        Subscription::none()
    }
}

fn main() -> iced::Result {
    init_logger();

    iced::application("azur-arknights-helper", App::update, App::view)
        .subscription(App::subscription)
        .run()
}

fn init_logger() {
    // let indicatif_layer = IndicatifLayer::new();

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("aah=info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(
            tracing_subscriber::fmt::layer()
                // .with_level(false)
                // .with_target(false)
                // .without_time()
                // .with_writer(indicatif_layer.get_stderr_writer()),
        )
        // .with(indicatif_layer)
        .init();
}