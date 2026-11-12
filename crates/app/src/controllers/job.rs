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

    /// Перед выходом ждём, пока отменённая задача освободит ресурсы.
    pub fn wait(&mut self) {
        if let Some(receiver) = self.0.take() {
            let _ = receiver.recv();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_arrives_once_and_frees_the_job() {
        let mut job = Job::new();
        assert!(!job.busy());
        job.spawn(|| 7);
        assert!(job.busy());
        let value = loop {
            match job.poll() {
                JobPoll::Done(value) => break value,
                JobPoll::Pending => std::thread::yield_now(),
                JobPoll::Lost => panic!("worker lost"),
            }
        };
        assert_eq!(value, 7);
        assert!(!job.busy());
        assert!(matches!(job.poll(), JobPoll::Pending));
    }

    #[test]
    fn panicking_worker_is_reported_as_lost() {
        let mut job: Job<u8> = Job::new();
        job.spawn(|| panic!("worker failed"));
        loop {
            match job.poll() {
                JobPoll::Lost => break,
                JobPoll::Pending => std::thread::yield_now(),
                JobPoll::Done(_) => panic!("no result expected"),
            }
        }
        assert!(!job.busy());
    }

    #[test]
    fn wait_blocks_until_the_worker_finishes() {
        let mut job = Job::new();
        job.spawn(|| std::thread::sleep(std::time::Duration::from_millis(30)));
        job.wait();
        assert!(!job.busy());
    }
}
