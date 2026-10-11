use crate::{i18n::tr, platform};
use z_core::preferences::Notifications;

#[derive(Clone, Copy)]
pub enum EventKind {
    Error,
    Strategy,
    Update
}

pub fn show(preferences: &Notifications, kind: EventKind, background: bool, message: &str) {
    let enabled = match kind {
        EventKind::Error => preferences.errors,
        EventKind::Strategy => preferences.strategy,
        EventKind::Update => preferences.updates,
    };
    if !enabled || !background {
        return;
    }
    let title = match kind {
        EventKind::Error => tr("zUI — ошибка", "zUI — error"),
        EventKind::Strategy => tr("zUI — стратегия", "zUI — strategy"),
        EventKind::Update => tr("zUI — обновление", "zUI — update"),
    };
    let message = message.to_string();
    std::thread::spawn(move || {
        let mut notification = notify_rust::Notification::new();
        notification.appname(platform::APP_NAME).summary(title).body(&message);

        #[cfg(windows)]
        notification.app_id(platform::APP_ID);
        let _ = notification.show();
    });
}
