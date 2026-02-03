use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use anyhow::Result;

/// Terminal events.
#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// Terminal tick.
    Tick,
    /// Key press.
    Key(KeyEvent),
    /// Terminal resize.
    Resize(u16, u16),
}

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    /// Event receiver channel.
    receiver: mpsc::Receiver<Event>,
    /// Event handler thread.
    handler: Option<thread::JoinHandle<()>>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();
        
        let handler = thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).expect("no events available") {
                    match event::read().expect("unable to read event") {
                        CrosstermEvent::Key(e) => {
                            if sender.send(Event::Key(e)).is_err() {
                                break;
                            }
                        }
                        CrosstermEvent::Resize(w, h) => {
                            if sender.send(Event::Resize(w, h)).is_err() {
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if sender.send(Event::Tick).is_err() {
                        break;
                    }
                    last_tick = Instant::now();
                }
            }
        });

        Self {
            receiver,
            handler: Some(handler),
        }
    }

    /// Receive the next event from the handler thread.
    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }
}

// Ensure the thread is joined on drop, making `handler` used.
impl Drop for EventHandler {
    fn drop(&mut self) {
        if let Some(handler) = self.handler.take() {
            // We can't easily signal the thread to stop without a channel in the other direction 
            // or an atomic bool, but since `sender` will be dropped when `EventHandler` is dropped 
            // (if we held sender, but we don't hold sender here, the thread holds sender).
            // Actually, the thread holds the *only* sender. When the receiver is dropped (here),
            // send() returns an error, breaking the loop. 
            // So joining here waits for that loop to break.
            let _ = handler.join();
        }
    }
}