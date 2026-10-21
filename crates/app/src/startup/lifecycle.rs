use super::StartupView;
use crate::{
    components::particle_background::ParticleField,
    controllers::{
        job::{Job, JobPoll},
        update::UpdateState,
    },
    i18n,
    root::RootView,
    state::{AppState, PreparationStage},
};
use gpui_kit::{component::*, *};
use std::{
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    time::{Duration, Instant},
};

/// Network latency must not hold the first usable screen indefinitely.
const UPDATE_WAIT: Duration = Duration::from_millis(1500);
const FRAME_INTERVAL: Duration = Duration::from_millis(33);
const INACTIVE_INTERVAL: Duration = Duration::from_millis(200);

impl StartupView {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        i18n::set_language(&z_core::preferences::Language::default());
        Theme::change(ThemeMode::Light, None, cx);
        // Before the normal root/tray exists, closing means exiting.
        window.on_window_should_close(cx, |_, cx| {
            cx.defer(|cx| cx.quit());
            true
        });
        // Start local I/O only after the loading screen has actually painted.
        cx.on_next_frame(window, |this, window, cx| this.begin(window, cx));
        Self {
            root: None,
            worker: Job::new(),
            progress: Arc::new(AtomicU8::new(PreparationStage::Settings as u8)),
            stage: PreparationStage::Settings,
            prepared: None,
            update_wait_started: None,
            particles: ParticleField::new(),
            failed: false,
        }
    }

    pub(super) fn begin(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.worker.busy() || self.root.is_some() {
            return;
        }
        self.failed = false;
        self.stage = PreparationStage::Settings;
        self.progress.store(self.stage as u8, Ordering::Relaxed);
        let progress = self.progress.clone();
        // AppState contains no GPUI entities or tray handles. Services are not
        // started here, so closing this window cannot orphan a launched process.
        self.worker.spawn(move || {
            AppState::prepare(|stage| {
                progress.store(stage as u8, Ordering::Relaxed);
            })
        });
        cx.notify();

        cx.spawn_in(window, async move |this, cx| {
            let mut interval = FRAME_INTERVAL;
            loop {
                cx.background_executor().timer(interval).await;
                match this.update_in(cx, |this, window, cx| this.tick(window, cx)) {
                    Ok(Some(next)) => interval = next,
                    _ => break,
                }
            }
        })
        .detach();
    }

    fn tick(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Option<Duration> {
        let next_stage = PreparationStage::from_progress(self.progress.load(Ordering::Relaxed));
        let mut changed = self.stage != next_stage;
        self.stage = next_stage;

        match self.worker.poll() {
            JobPoll::Done(state) => {
                self.prepared = Some(state);
                self.update_wait_started = Some(Instant::now());
                changed = true;
            }
            JobPoll::Lost => {
                self.failed = true;
                cx.notify();
                return None;
            }
            JobPoll::Pending => {}
        }

        if let Some(state) = &mut self.prepared {
            state.update.poll();
            state.zapret.poll();
            let checking = matches!(state.update.state, UpdateState::Checking)
                || state.zapret.checking_update();
            let expired = self
                .update_wait_started
                .is_some_and(|started| started.elapsed() >= UPDATE_WAIT);
            if !checking || expired {
                let state = self.prepared.take().expect("prepared application state");
                // RootView keeps the existing onboarding, what's-new, tray and
                // close behavior, and starts configured services exactly once.
                self.root = Some(cx.new(|cx| RootView::new(state, window, cx)));
                self.particles.clear();
                cx.notify();
                return None;
            }
        }

        let active = window.is_window_active();
        let animate = active && !cx.reduce_motion();
        self.particles.advance(animate);
        if animate || changed {
            cx.notify();
        }
        Some(if active {
            FRAME_INTERVAL
        } else {
            INACTIVE_INTERVAL
        })
    }
}
