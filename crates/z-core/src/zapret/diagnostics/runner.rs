//! Поочерёдный тест стратегий с отменой и уборкой процессов.
use super::probe::probe;
use super::{Options, Progress, Score};
use crate::{
    log::Log,
    zapret::{Strategy, Zapret},
};
use std::{
    path::Path,
    sync::atomic::Ordering,
    thread::sleep,
    time::{Duration, Instant},
};

/// Сортировка по числу открывшихся адресов, затем по средней задержке по возрастанию.
pub fn sort_scores(v: &mut [Score]) {
    v.sort_by(|a, b| b.passed.cmp(&a.passed).then(a.avg_ms.cmp(&b.avg_ms)));
}

/// Тестирует стратегии по очереди в отдельном процессе winws.
///
/// Вызывать из рабочего потока: функция ждёт сетевых проверок и запуска
/// процесса. При отмене возвращает готовые оценки и останавливает свой процесс.
/// Перед запуском вызывающий код должен остановить свой обычный процесс Zapret.
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
    // Узел ищем один раз, пока обычный Zapret остановлен: адрес не зависит от стратегии.
    let video = if opts.video {
        let node = rt.block_on(async {
            let client = crate::net::client(Duration::from_secs(5), Duration::from_secs(8)).ok()?;
            crate::net::video_node(&client).await
        });
        if node.is_none() {
            log.add("[test] Видеоузел YouTube не найден, проверяем без него");
        }
        node
    } else {
        None
    };

    for s in strategies {
        if progress.cancel.load(Ordering::SeqCst) {
            break;
        }
        let Some(score) = check_one(&rt, &mut zapret, dir, &s, &opts.targets, video, progress)
        else {
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
    video: Option<std::net::SocketAddr>,
    progress: &Progress,
) -> Option<Score> {
    let mut failed = targets.to_vec();
    failed.extend(video.map(super::probe::video_label));
    let mut score = Score {
        file_name: s.file_name.clone(),
        name: s.name.clone(),
        passed: 0,
        total: failed.len(),
        avg_ms: 0,
        failed,
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
    // winws, упавший на аргументах стратегии, ничего не обходит. Если сайты при этом
    // открываются (через VPN или вообще без блокировки), стратегия всё равно не годится.
    if !zapret.is_running() {
        score.error = Some(WINWS_EXITED.into());
        return Some(score);
    }
    let result = rt.block_on(async {
        tokio::select! {
            result = probe(targets, video) => Some(result),
            _ = wait_for_cancel(progress) => None,
        }
    });
    let survived = zapret.is_running();
    zapret.stop();
    if result.is_some() && !survived {
        score.error = Some(WINWS_EXITED.into());
        return Some(score);
    }
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

const WINWS_EXITED: &str = "winws завершился во время проверки: стратегия не работает";

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
