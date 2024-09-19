use std::thread::sleep;
use std::time::Duration;

use aah_core::task::TaskEvt;
use aah_core::AAH;
use iced::futures::SinkExt;
use iced::futures::{channel::mpsc, Stream};
use iced::stream::channel;
use tracing::{error, info};

// pub enum AAHCoreEvent {
//     Ready(mpsc::Sender<CoreCommand>),
//     TaskEvt(TaskEvt),
// }

// pub enum CoreCommand {

// }

// pub fn aah_core() -> impl Stream<Item = AAHCoreEvent> {
//     channel(100, |mut output| async move {
//         let (sender, mut receiver) = mpsc::channel(100);
//         output.send(AAHCoreEvent::Ready(sender)).await.unwrap();

//         let mut aah: Option<AAH> = None;
//         loop {

//         }
//     })
// }

pub enum Event {
    Ready(mpsc::UnboundedSender<Input>),
    ListeningToTaskEvt,
    TaskEvt(TaskEvt),
}

pub enum Input {
    StartListenToTaskEvt(async_channel::Receiver<TaskEvt>),
}

pub fn task_evt_listener() -> impl Stream<Item = Event> {
    channel(0, |mut output| async move {
        // Create channel
        let (sender, mut receiver) = mpsc::unbounded();

        // Send the sender back to the application
        output.send(Event::Ready(sender)).await.unwrap();

        let mut task_evt_receiver: Option<async_channel::Receiver<TaskEvt>> = None;
        loop {
            use iced::futures::StreamExt;

            if let Some(task_evt_receiver) = &task_evt_receiver {
                match task_evt_receiver.recv().await {
                    Ok(task_evt) => output.send(Event::TaskEvt(task_evt)).await.unwrap(),
                    Err(e) => error!("Failed to receive task event: {}", e),
                }
            } else {
                match receiver.select_next_some().await {
                    Input::StartListenToTaskEvt(receiver) => {
                        task_evt_receiver = Some(receiver);
                        info!("setted task_evt_receiver");
                        output.send(Event::ListeningToTaskEvt).await.unwrap();
                    }
                }
            }
        }
    })
}
