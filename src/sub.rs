use git2::Repository;
use iced::{futures::{SinkExt, Stream}, stream};

#[derive(Debug, Clone)]
pub enum Event {
    InitializingResource,
    ResourceInitialized,
}

pub fn init_resource() -> impl Stream<Item = Event> {
    println!("init_resource");
    stream::channel(100, |mut output| async move {
        output.send(Event::InitializingResource).await.unwrap();

    })
}