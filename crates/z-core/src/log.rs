use std::sync::{Arc, Mutex};

const MAX_LINES: usize = 1000;

#[derive(Clone, Default)]
pub struct Log(Arc<Mutex<Vec<String>>>);

impl Log {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&self, msg: impl AsRef<str>) {
        if let Ok(mut l) = self.0.lock() {
            l.push(msg.as_ref().to_string());
            if l.len() > MAX_LINES {
                let extra = l.len() - MAX_LINES;
                l.drain(..extra);
            }
        }
    }

    pub fn get(&self) -> Vec<String> {
        self.0.lock().map(|l| l.clone()).unwrap_or_default()
    }

    pub fn clear(&self) {
        if let Ok(mut l) = self.0.lock() {
            l.clear();
        }
    }

}
