# Startup screen

The first window shows the application logo, version, current preparation stage, a short
progress line under the name and a canvas of glyph streams behind them.

## Lifecycle

- `main.rs` creates `StartupView` inside the usual GPUI Kit root.
- After its first painted frame, `startup/lifecycle.rs` starts a background worker.
- `AppState::prepare` reads settings, loads language packs and prepares local controllers,
  then starts the existing application and Zapret update checks.
- GPUI polls the prepared state while those checks finish. The network grace period is
  at most 1.5 seconds after local preparation, with no minimum splash duration.
- A finished check or an expired grace period shows the update result for 0.9 seconds:
  a new version, no updates, or that the check continues in the background.
- Then, if zUI runs without administrator rights, onboarding is done and the
  `ask_elevation` setting is on, the screen offers to restart as administrator or continue.
  Otherwise, or after "Continue", the same state goes to `RootView`.
  Pending checks continue through the existing polling and update banners.
- RootView retains onboarding, what's-new, service startup, tray and close preferences.

The worker does not launch services, install updates or change DNS. Services start once
the prepared state reaches RootView. Closing during preparation exits; no process can
be orphaned by a worker that finishes after the window closes. A disconnected preparation
worker exposes a retry button instead of leaving an indefinite loader.

## Visual effect

`components/signal_field.rs` paints five streams of monospace glyphs (` .·:-=+x#`) that
come in from the window edges, meet in the logo and spread out again. Brighter cells use
denser glyphs; pulses run along each stream toward the logo. The nine glyphs are shaped
once per frame and then only placed. The cursor pulls nearby streams; the pull, cursor
position, speed and dimming all ease toward their targets instead of jumping. While the
elevation question waits for an answer, the streams slow down and dim. The update result
briefly turns the densest glyphs into the success color.

The startup timer drives motion at approximately 30 fps, backs off when the window is
inactive, and respects GPUI's reduced-motion preference. The timer ends at handoff.
There is no such animation in the main application.

All preparation status and error strings belong to the RU/EN Fluent catalogs.

The first frame uses an isolated bundled localization fallback via `startup_tr`.
It never initializes the main application catalog before `i18n::initialize` has had
an opportunity to load installed language packs from the runtime directory.