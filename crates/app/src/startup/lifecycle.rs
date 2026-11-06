use super::{Phase, StartupView};
use crate::{
    components::signal_field::SignalField,
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

/// Задержка сети не должна бесконечно задерживать первый рабочий экран.
const UPDATE_WAIT: Duration = Duration::from_millis(1500);
const FRAME_INTERVAL: Duration = Duration::from_millis(33);
const INACTIVE_INTERVAL: Duration = Duration::from_millis(200);
/// Сколько держим на экране итог проверки обновлений.
const RESULT_HOLD: Duration = Duration::from_millis(900);

impl StartupView {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        i18n::set_language(&z_core::preferences::Language::default());
        // Настройки ещё не загружены, поэтому берём тему Windows, чтобы не мигнуть белым в тёмной теме.
        let mode = if crate::platform::system_dark() {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };
        Theme::change(mode, None, cx);
        // Пока нет основного окна и трея, закрытие означает выход.
        window.on_window_should_close(cx, |_, cx| {
            cx.defer(|cx| cx.quit());
            true
        });
        // Локальный ввод-вывод начинаем только после того, как экран загрузки отрисован.
        cx.on_next_frame(window, |this, window, cx| this.begin(window, cx));
        Self {
            root: None,
            worker: Job::new(),
            progress: Arc::new(AtomicU8::new(PreparationStage::Settings as u8)),
            stage: PreparationStage::Settings,
            prepared: None,
            update_wait_started: None,
            field: SignalField::new(),
            phase: Phase::Preparing,
            update_note: None,
            failed: false,
        }
    }

    pub(super) fn begin(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.worker.busy() || self.root.is_some() {
            return;
        }
        self.failed = false;
        self.phase = Phase::Preparing;
        self.update_note = None;
        self.stage = PreparationStage::Settings;
        self.progress.store(self.stage as u8, Ordering::Relaxed);
        let progress = self.progress.clone();
        // AppState не содержит сущностей GPUI и дескрипторов трея. Сервисы здесь
        // не запускаются, поэтому закрытие окна не оставит процесс без владельца.
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
        if self.root.is_some() {
            return None;
        }
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
            match self.phase {
                Phase::Preparing => {
                    let checking = matches!(state.update.state, UpdateState::Checking)
                        || state.zapret.checking_update();
                    let expired = self
                        .update_wait_started
                        .is_some_and(|started| started.elapsed() >= UPDATE_WAIT);
                    if !checking || expired {
                        self.update_note = Some(match &state.update.state {
                            UpdateState::Available(version) => i18n::tr_args(
                                "startup-update-available",
                                &[("version", version.clone().into())],
                            ),
                            // Сеть медленная: проверка доделается в фоне, экран не ждёт.
                            UpdateState::Checking => i18n::tr("startup-update-background"),
                            _ => i18n::tr("startup-update-none"),
                        });
                        self.field.flash();
                        self.phase = Phase::UpdateResult(Instant::now());
                        changed = true;
                    }
                }
                Phase::UpdateResult(since) if since.elapsed() >= RESULT_HOLD => {
                    if Self::should_ask_elevation(state) {
                        self.phase = Phase::AskElevation;
                        self.field.set_waiting(true);
                        changed = true;
                    } else {
                        self.open_app(window, cx);
                        return None;
                    }
                }
                _ => {}
            }
        }

        let active = window.is_window_active();
        let animate = active && !cx.reduce_motion();
        self.field.advance(animate);
        if animate || changed {
            cx.notify();
        }
        Some(if active {
            FRAME_INTERVAL
        } else {
            INACTIVE_INTERVAL
        })
    }

    /// Спрашиваем про права только тех, кто уже прошёл мастер: в мастере есть свой шаг.
    fn should_ask_elevation(state: &AppState) -> bool {
        state.settings.onboarded && state.settings.ask_elevation && !z_core::dns::is_elevated()
    }

    /// Передаёт подготовленное состояние обычному окну приложения.
    pub(super) fn open_app(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(state) = self.prepared.take() else {
            return;
        };
        // RootView сохраняет мастер первого запуска, «Что нового», трей и
        // поведение закрытия и запускает настроенные сервисы ровно один раз.
        self.root = Some(cx.new(|cx| RootView::new(state, window, cx)));
        cx.notify();
    }

    /// Сохраняет настройки и перезапускает zUI через запрос UAC.
    pub(super) fn restart_elevated(&mut self, cx: &mut Context<Self>) {
        if self.prepared.as_mut().is_some_and(AppState::elevate) {
            cx.quit();
        }
    }

    pub(super) fn pointer_moved(&mut self, position: Point<Pixels>) {
        self.field.pointer_moved(position);
    }

    pub(super) fn pointer_left(&mut self) {
        self.field.pointer_left();
    }
}
