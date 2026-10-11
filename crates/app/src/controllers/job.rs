use std::{
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
};

pub enum JobPoll<T> {
    Pending,
    Done(T),
    Lost, // поток умер, не прислав результат
}

pub struct Job<T>(Option<Receiver<T>>);

impl<T: Send + 'static> Job<T> {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn busy(&self) -> bool {
        self.0.is_some()
    }

    pub fn spawn(&mut self, f: impl FnOnce() -> T + Send + 'static) {
        let (tx, rx) = mpsc::channel();
        self.0 = Some(rx);
        thread::spawn(move || {
            let _ = tx.send(f());
        });
    }

    pub fn poll(&mut self) -> JobPoll<T> {
        let Some(rx) = &self.0 else {
            return JobPoll::Pending;
        };
        match rx.try_recv() {
            Ok(v) => {
                self.0 = None;
                JobPoll::Done(v)
            }
            Err(TryRecvError::Disconnected) => {
                self.0 = None;
                JobPoll::Lost
            }
            Err(TryRecvError::Empty) => JobPoll::Pending,
        }
    }

    /// Wait for a cancelled worker to release its resources before app exit.
    pub fn wait(&mut self) {
        if let Some(receiver) = self.0.take() {
            let _ = receiver.recv();
        }
    }
}
