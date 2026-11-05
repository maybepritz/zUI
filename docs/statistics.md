# Statistics

The statistics screen presents saved diagnostic results, lifetime counters and recent events.
It does not perform network requests while rendering and does not treat an old test as live
connection health.

## Data and persistence

`crates/z-core/src/statistics.rs` owns the serialized model in `runtime/statistics.json`.
New fields default to an empty history and no diagnostic snapshot, preserving existing counters.
Only the last test and 100 most recent events are retained; the screen shows the newest 20.
An existing successful-test timestamp remains visible until a detailed result is available.
History is local and follows the existing statistics save lifecycle.

A diagnostic snapshot records whether the run completed, how many strategies were tested,
and the best usable result (its name, file name, reachable target count and mean latency).
A cancelled run is explicitly marked incomplete. Targets are URLs, not distinct services:
several targets may belong to the same service.
Total uptime includes all application sessions, including time spent hidden.

## Responsibilities

- `state/testing.rs`: capture the final diagnostic snapshot.
- `state/strategies.rs`: record strategy changes.
- `state/background.rs`: record watchdog recovery attempts and persist statistics.
- `views/statistics/`: layout, relative-time formatting, summary and event rows.
- `components/cards/stat_card.rs`: reusable metric presentation.

Events store typed data rather than translated sentences. Their descriptions are formatted
using the currently selected language. Add matching keys and variables to both bundled
`locales/en/main.ftl` and `locales/ru/main.ftl` when extending this screen.

## Service availability

`crates/z-core/src/health.rs` keeps the last day of service checks in `health.json` next to
`statistics.json`. Checks run every minute while the bypass is running or automatic switching
is on; results are folded into 15-minute buckets (successes, failures, summed latency).
A check without internet is skipped. When a service opens again after failing, the outage is
recorded as a `ServiceDown` event in the statistics history. Losing `health.json` only resets
the chart, so it has no backup.
