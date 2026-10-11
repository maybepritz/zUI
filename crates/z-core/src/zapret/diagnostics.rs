use super::{Strategy, Zapret};
use crate::log::Log;
use anyhow::Result;
use std::{
    path::Path,
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread::sleep,
    time::{Duration, Instant},
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

pub fn sort_scores(v: &mut [Score]) {
    v.sort_by(|a, b| b.passed.cmp(&a.passed).then(a.avg_ms.cmp(&b.avg_ms)));
}

pub fn run(
    dir: &Path,
    strategies: Vec<Strategy>,
    opts: Options,
    log: Log,
    progress: &Progress,
) -> Vec<Score> {
    if !crate::dns::is_elevated() {
        log.add("[test] Нужны права администратора");
        return Vec::new();
    }
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            log.add(format!("[test] Не удалось создать runtime: {e}"));
            return Vec::new();
        }
    };
    let mut zapret = Zapret::new(log.clone());

    for s in strategies {
        if progress.cancel.load(Ordering::SeqCst) {
            break;
        }
        let Some(score) = check_one(&rt, &mut zapret, dir, &s, &opts.targets, progress) else {
            break;
        };
        log.add(match &score.error {
            Some(e) => format!("[test] {}: не запустилась: {e}", score.name),
            None => format!(
                "[test] {}: {}/{}, {} мс",
                score.name, score.passed, score.total, score.avg_ms
            ),
        });
        let ok = score.is_ok();
        if let Ok(mut live) = progress.live.lock() {
            live.push(score);
        }
        progress.done.fetch_add(1, Ordering::SeqCst);
        if ok && opts.stop_on_first_ok {
            break;
        }
    }
    zapret.stop();

    let mut all = progress.live.lock().map(|l| l.clone()).unwrap_or_default();
    sort_scores(&mut all);
    all
}

fn check_one(
    rt: &tokio::runtime::Runtime,
    zapret: &mut Zapret,
    dir: &Path,
    s: &Strategy,
    targets: &[String],
    progress: &Progress,
) -> Option<Score> {
    let mut score = Score {
        file_name: s.file_name.clone(),
        name: s.name.clone(),
        passed: 0,
        total: targets.len(),
        avg_ms: 0,
        failed: targets.to_vec(),
        error: None,
    };
    if let Err(e) = zapret.start(&dir.join("bin"), &dir.join("lists"), &s.raw_args) {
        score.error = Some(format!("{e:#}"));
        return Some(score);
    }
    if !wait_unless_cancelled(Duration::from_secs(3), progress) {
        zapret.stop();
        return None;
    }
    let result = rt.block_on(async {
        tokio::select! {
            result = probe(targets) => Some(result),
            _ = wait_for_cancel(progress) => None,
        }
    });
    zapret.stop();
    match result? {
        Ok(results) => {
            let times: Vec<u64> = results.iter().filter_map(|(_, t)| *t).collect();
            score.passed = times.len();
            score.avg_ms = if times.is_empty() {
                0
            } else {
                times.iter().sum::<u64>() / times.len() as u64
            };
            score.failed = results
                .into_iter()
                .filter(|(_, t)| t.is_none())
                .map(|(url, _)| url)
                .collect();
        }
        Err(e) => score.error = Some(format!("{e:#}")),
    }
    wait_unless_cancelled(Duration::from_secs(1), progress); // освободить WinDivert
    Some(score)
}

fn wait_unless_cancelled(duration: Duration, progress: &Progress) -> bool {
    let start = Instant::now();
    while start.elapsed() < duration {
        if progress.cancel.load(Ordering::SeqCst) {
            return false;
        }
        sleep(Duration::from_millis(50).min(duration.saturating_sub(start.elapsed())));
    }
    !progress.cancel.load(Ordering::SeqCst)
}

async fn wait_for_cancel(progress: &Progress) {
    while !progress.cancel.load(Ordering::SeqCst) {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn probe(targets: &[String]) -> Result<Vec<(String, Option<u64>)>> {
    // новый клиент на каждую проверку: без keep-alive от прошлой стратегии
    let client = reqwest::Client::builder()
        .user_agent("zapret-ui/0.1")
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(8))
        .build()?;
    // Dropping JoinSet aborts outstanding probes when the user cancels the test.
    let mut handles = tokio::task::JoinSet::new();
    for url in targets {
        let (client, url) = (client.clone(), url.clone());
        handles.spawn(async move {
            let started = Instant::now();
            let ok = client.get(&url).send().await.is_ok_and(|response| response.status().is_success());
            (url, ok.then(|| started.elapsed().as_millis() as u64))
        });
    }
    let mut out = Vec::new();
    while let Some(result) = handles.join_next().await {
        if let Ok(r) = result {
            out.push(r);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn score(name: &str, passed: usize, avg_ms: u64) -> Score {
        Score {
            file_name: format!("{name}.bat"),
            name: name.into(),
            passed,
            total: 5,
            avg_ms,
            failed: vec![],
            error: None,
        }
    }

    #[test]
    fn sorts_by_passed_then_latency() {
        let mut v = vec![score("a", 3, 100), score("b", 5, 400), score("c", 5, 200)];
        sort_scores(&mut v);
        let names: Vec<_> = v.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["c", "b", "a"]);
    }

    #[test]
    fn ok_requires_all_targets_and_no_error() {
        assert!(score("a", 5, 10).is_ok());
        assert!(!score("a", 4, 10).is_ok());
        let mut s = score("a", 5, 10);
        s.error = Some("x".into());
        assert!(!s.is_ok());
    }

    #[test]
    fn cancelled_initialization_does_not_wait_for_the_delay() {
        let progress = Progress::default();
        progress.cancel.store(true, Ordering::SeqCst);
        assert!(!wait_unless_cancelled(Duration::from_secs(60), &progress));
    }

    #[tokio::test]
    async fn cancellation_interrupts_pending_work() {
        let progress = Progress::default();
        progress.cancel.store(true, Ordering::SeqCst);
        tokio::time::timeout(Duration::from_secs(1), async {
            tokio::select! {
                _ = std::future::pending::<()>() => panic!("pending work completed"),
                _ = wait_for_cancel(&progress) => {}
            }
        })
        .await
        .expect("cancellation must not wait for a network timeout");
    }
}
