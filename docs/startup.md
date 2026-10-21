# Startup screen

The first window contains a white, monochrome startup screen with the application logo,
version, current preparation stage and a lightweight particle canvas.

## Lifecycle

- `main.rs` creates `StartupView` inside the usual GPUI Kit root.
- After its first painted frame, `startup/lifecycle.rs` starts a background worker.
- `AppState::prepare` reads settings, loads language packs and prepares local controllers,
  then starts the existing application and Zapret update checks.
- GPUI polls the prepared state while those checks finish. The network grace period is
  at most 1.5 seconds after local preparation, with no minimum splash duration.
- A finished check or an expired grace period hands the same state to `RootView`.
  Pending checks continue through the existing polling and update banners.
- RootView retains onboarding, what's-new, service startup, tray and close preferences.

The worker does not launch services, install updates or change DNS. Services start once
the prepared state reaches RootView. Closing during preparation exits; no process can
be orphaned by a worker that finishes after the window closes. A disconnected preparation
worker exposes a retry button instead of leaving an indefinite loader.

## Visual effect

`components/particle_background.rs` paints 48 drifting gray dots in one GPUI canvas.
Nearby particles move away from the cursor and become darker. A capped 36-point cursor
trail fades within 550 ms. The central text area stays quieter than the surrounding field.

The startup timer drives motion at approximately 30 fps, backs off when the window is
inactive, and respects GPUI's reduced-motion preference. The timer ends at handoff and
clears particle data. There is no particle animation in the main application.

All preparation status and error strings belong to the RU/EN Fluent catalogs.

The first frame uses an isolated bundled localization fallback via `startup_tr`.
It never initializes the main application catalog before `i18n::initialize` has had
an opportunity to load installed language packs from the runtime directory.