use crossterm::event::Event as CrosstermEvent;
use crossterm::event::EventStream;
use futures::{StreamExt, future::FutureExt};
use tokio::{
    sync::mpsc::{UnboundedReceiver, unbounded_channel},
    time,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub enum Event {
    Draw,
    Key(CrosstermEvent),
}

pub struct EventLoop {
    // Used by the application to listen to event
    pub event_rx: UnboundedReceiver<Event>,
    // Cancelation token that is used to cancel the event loop
    pub abort: CancellationToken,
}

impl EventLoop {
    pub fn start() -> Self {
        let (event_tx, event_rx) = unbounded_channel::<Event>();
        let abort_token = CancellationToken::new();
        // Keep a reference and listen on it
        let cloned_abort = abort_token.clone();

        // Initial draw event
        // This is sent so we start drawring directly instead of waiting 1 sec at the beginning
        event_tx
            .send(Event::Draw)
            .expect("Unexpected error when trying to send a draw event");
        tokio::spawn(async move {
            let mut stream = EventStream::new();
            loop {
                tokio::select! {
                    // Send a draw event every second
                    _ = time::sleep(time::Duration::from_secs(1)) => {
                        event_tx.send(Event::Draw).expect("Unexpected error when trying to send a draw event");
                    }
                    // Listen on the crossterm event stream and send events
                    key = stream.next().fuse() => {
                        if let Some(Ok(key)) = key {
                            event_tx.send(Event::Key(key)).expect("Unexpected error when trying to send a key event");
                        }
                    }
                    // Check if the token was canceled
                    _ = cloned_abort.cancelled() => {
                        break;
                    }
                }
            }
        });

        Self {
            event_rx,
            abort: abort_token,
        }
    }
}
