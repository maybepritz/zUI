//! Inputs, progress and scores shared by diagnostics workers and the UI.
use std::sync::{
    Mutex,
    atomic::{AtomicBool, AtomicUsize},
};

pub const DEFAULT_TARGETS: [&str; 12] = [
    "https://discord.com",
    "https://gateway.discord.gg",
    "https://cdn.discordapp.com",
    "https://updates.discord.com",
    "https://www.youtube.com",
    "https://youtu.be",
    "https://i.ytimg.com",
    "https://redirector.googlevideo.com",
    "https://www.google.com",
    "https://www.gstatic.com",
    "https://www.cloudflare.com",
    "https://cdnjs.cloudflare.com",
];

#[derive(Clone, Debug)]
pub struct Score {
    pub file_name: String,
    pub name: String,
    pub passed: usize,
    pub total: usize,
    pub avg_ms: u64,
    pub failed: Vec<String>,
    pub error: Option<String>,
}

impl Score {
    pub fn is_ok(&self) -> bool {
        self.error.is_none() && self.total > 0 && self.passed == self.total
    }
}

pub struct Options {
    pub targets: Vec<String>,
    pub stop_on_first_ok: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            targets: DEFAULT_TARGETS.iter().map(|s| s.to_string()).collect(),
            stop_on_first_ok: false,
        }
    }
}

#[derive(Default)]
pub struct Progress {
    pub done: AtomicUsize,
    pub cancel: AtomicBool,
    pub live: Mutex<Vec<Score>>,
}
