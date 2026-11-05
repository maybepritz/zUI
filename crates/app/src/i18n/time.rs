use crate::i18n::{tr, tr_args};

pub(crate) fn duration(secs: u64) -> String {
    let (d, h, m) = (secs / 86400, secs / 3600 % 24, secs / 60 % 60);
    if d > 0 {
        tr_args(
            "duration-days-hours",
            &[("days", d.into()), ("hours", h.into())],
        )
    } else if h > 0 {
        tr_args(
            "duration-hours-minutes",
            &[("hours", h.into()), ("minutes", m.into())],
        )
    } else {
        tr_args("duration-minutes", &[("minutes", m.into())])
    }
}

pub(crate) fn ago(timestamp: u64, now: u64) -> String {
    let secs = now.saturating_sub(timestamp);
    if secs < 60 {
        tr("ui-just-now")
    } else {
        tr_args("time-ago", &[("duration", duration(secs).into())])
    }
}
