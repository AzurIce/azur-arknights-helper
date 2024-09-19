use aah_core::task::TaskEvt;
use iced::futures::SinkExt;
use iced::futures::{channel::mpsc, Stream};
use iced::stream::channel;

pub enum Event {
    Ready(mpsc::Sender<Input>),
    ListeningToTaskEvt,
    TaskEvt(TaskEvt),
}

pub enum Input {
    StartListenToTaskEvt(async_channel::Receiver<TaskEvt>),
}

pub fn task_evt_listener() -> impl Stream<Item = Event> {
    channel(100, |mut output| async move {
        // Create channel
        let (sender, mut receiver) = mpsc::channel(100);

        // Send the sender back to the application
        output.send(Event::Ready(sender)).await.unwrap();

        let mut task_evt_receiver: Option<async_channel::Receiver<TaskEvt>> = None;
        loop {
            use iced::futures::StreamExt;

            if let Some(task_evt_receiver) = &task_evt_receiver {
                match task_evt_receiver.recv().await {
                    Ok(task_evt) => output.send(Event::TaskEvt(task_evt)).await.unwrap(),
                    Err(e) => tracing::error!("Failed to receive task event: {}", e),
                }
            } else {
                match receiver.select_next_some().await {
                    Input::StartListenToTaskEvt(receiver) => {
                        task_evt_receiver = Some(receiver);
                        output.send(Event::ListeningToTaskEvt).await.unwrap();
                    }
                }
            }
        }
    })
}
