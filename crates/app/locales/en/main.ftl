# zUI UI messages. Keep message IDs and variable names unchanged.

# Profiles and window-close dialogs
profiles-title = Profiles
profiles-hint = Save your strategy, DNS, service startup and automation settings as a named profile.
profiles-empty = No profiles yet. Configure the app and save your first profile.
profile-create = Save current configuration
profile-save = Save profile
profile-rename = Rename
profile-update = Update from current settings
profile-delete = Delete profile
profile-apply = Apply
profile-active = Applied
profile-name = Profile name
profile-name-placeholder = For example: Home or Work
profile-create-hint = Save the selected strategy and current startup/automation settings. Applying a profile starts or stops services according to its startup switches.
profile-rename-hint = The name changes; saved settings and profile ID stay the same.
profile-update-hint = Replace the settings in “{ $name }” with the current configuration?
profile-delete-hint = Delete “{ $name }”? Running services and the current configuration will stay as they are.
profile-dns-label = DNS when applying this profile
profile-dns-current = Save current preset
profile-dns-keep = Keep current DNS
profile-dns-automatic = Automatic (DHCP)
profile-details = Strategy: { $strategy } · DNS: { $dns }
profile-services = Services on apply and startup: Zapret { $zapret }, Telegram { $telegram }
profile-on = on
profile-off = off
profile-saved = Profile saved.
profile-renamed = Profile renamed.
profile-updated = Profile updated. Apply it to activate the saved configuration.
profile-deleted = Profile deleted.
profile-applying = Applying “{ $name }”… Waiting for DNS.
profile-applied = “{ $name }” applied.
profile-modified = The configuration changed. No saved profile is currently marked as applied.
profile-apply-failed = Profile not fully applied: { $error }. Some settings may have changed; review the current configuration.
profile-error-name = Enter a profile name.
profile-error-long-name = Use no more than 80 characters for the name.
profile-error-duplicate = A profile with this name already exists.
profile-error-busy = Wait for DNS, testing or installation to finish.
profile-error-strategy = The saved strategy is unavailable. Install Zapret or update the profile.
profile-error-missing = This profile no longer exists.
profile-error-save = Could not save the profile. Check the event log and folder permissions.
profile-error-dns = The saved DNS preset is unavailable. Update the profile.
profile-error-admin = Run zUI as administrator to apply DNS or start Zapret.
profile-error-telegram = Install Telegram Proxy before applying this profile.
close-dialog-title = Close zUI?
close-dialog-hint = Hide keeps zUI and services running in the tray. Exit stops Zapret and Telegram Proxy and closes the application.
close-hide = Hide to tray
close-exit = Exit application
close-remember = Remember my choice
close-remember-setting = Use the saved close action without asking
close-error-save = Could not remember your choice. Check the event log and try again.

# Used in: views/strategies/filters.rs, root/strategy_filters.rs
strategy-list-title = Available strategies
strategy-search-label = Search strategies
strategy-search-placeholder = Search by name or file…
strategy-filter-favorites = Favorites only
strategy-filter-reset = Reset filters
strategy-filter-count = Showing { $shown } of { $total }
strategy-list-empty = No strategies available
strategy-list-empty-hint = Install Zapret or check its files on the Home page.
strategy-open-home = Open Home
strategy-favorites-empty = No favorite strategies
strategy-favorites-empty-hint = Mark strategies with a star or reset the filters.
strategy-search-empty = No strategies found
strategy-search-empty-hint = Try a different query or reset the filters.

# Used in: views/strategies/card.rs
favorite-add = Add to favorites
favorite-remove = Remove from favorites
favorite-label = Favorite

# Used in: notifications.rs
ui-zui-error = zUI — error

# Used in: notifications.rs
ui-zui-strategy = zUI — strategy

# Used in: notifications.rs
ui-zui-update = zUI — update

# Used in: components/log_preview.rs
ui-no-events-yet-start-zapret-to-begin = No events yet. Start Zapret to begin.

# Used in: components/log_preview.rs
ui-recent-events = Recent events

# Used in: components/log_preview.rs
ui-all-logs = All logs →

# Used in: root/navigation.rs
ui-home = Home

# Used in: root/navigation.rs
ui-strategies = Strategies

# Used in: root/navigation.rs, views/statistics.rs
ui-statistics = Statistics

# Used in: root/navigation.rs
ui-logs = Logs

# Used in: root/navigation.rs, views/settings/mod.rs
ui-settings = Settings

# Used in: state/background.rs
ui-winws-exited-and-was-restarted = winws exited and was restarted

# Used in: state/background.rs
ui-could-not-restart-winws-automatic-retries-are = Could not restart winws. Automatic retries are limited to three attempts

# Used in: state/reports.rs
ui-report-saved-review-it-before-publishing = Report saved. Review it before publishing

# Used in: state/settings.rs
ui-settings-saved = Settings saved

# Used in: state/testing.rs
ui-could-not-start-the-selected-strategy-the = Could not start the selected strategy; the previous one was restored

# Used in: state/testing.rs
ui-could-not-restore-zapret-after-testing-see = Could not restore Zapret after testing. See the log

# Used in: state/testing.rs
ui-a-better-strategy-was-found-and-applied = A better strategy was found and applied

# Used in: state/testing.rs
ui-a-better-strategy-is-available-on-the = A better strategy is available on the Strategies page

# Used in: state/testing.rs
ui-test-complete-current-strategy-retained = Test complete. Current strategy retained

# Used in: state/testing.rs
ui-test-cancelled-previous-state-restored = Test cancelled. Previous state restored

# Used in: views/home.rs
ui-not-selected = Not selected

# Used in: views/home.rs
ui-stop-zapret = Stop Zapret

# Used in: views/home.rs
ui-start-zapret = Start Zapret

# Used in: views/home.rs
ui-current-configuration = Current configuration

# Used in: views/home.rs
ui-strategy = Strategy:

# Used in: views/home.rs
ui-choose-another = Choose another

# Used in: views/home.rs
ui-dns-server = DNS server:

# Used in: views/home.rs
ui-configure-dns = Configure DNS

# Used in: views/home.rs
ui-configure-proxy = Configure proxy

# Used in: views/logs.rs
ui-no-events-yet-start-zapret-on-the = No events yet. Start Zapret on the Home page to view events.

# Used in: views/logs.rs
ui-event-log = Event log

# Used in: views/logs.rs
ui-clear-log = Clear log

# Used in: views/statistics.rs
ui-just-now = just now

# Used in: views/statistics.rs
ui-not-yet = Not yet

# Used in: views/statistics.rs
ui-zui-uptime = zUI uptime

# Used in: views/statistics.rs
ui-last-successful-test = Last successful test

# Used in: views/statistics.rs
ui-strategy-changes = Strategy changes

# Used in: views/statistics.rs
ui-winws-restarts = winws restarts

# Used in: components/cards/banner.rs
ui-zapret-is-not-installed = Zapret is not installed

# Used in: components/cards/banner.rs
ui-the-latest-zapret-discord-youtube-release-will = The latest zapret-discord-youtube release will be downloaded into the runtime folder.

# Used in: components/cards/banner.rs
ui-installing = Installing…

# Used in: components/cards/banner.rs
ui-install-zapret = Install Zapret

# Used in: components/cards/status_card.rs
ui-running = ● RUNNING

# Used in: components/cards/status_card.rs
ui-stopped = ○ STOPPED

# Used in: components/cards/update_banner.rs
ui-later = Later

# Used in: components/cards/update_banner.rs
ui-update = Update

# Used in: components/cards/update_banner.rs
ui-installing-update = Installing update…

# Used in: components/cards/update_banner.rs
ui-restart = Restart

# Used in: components/cards/update_banner.rs
ui-close = Close

# Used in: components/cards/update_banner.rs
ui-updating-zapret = Updating Zapret…

# Used in: root/handlers/support.rs
ui-report-saved = Report saved

# Used in: root/handlers/support.rs
ui-could-not-save-the-report = Could not save the report

# Used in: root/handlers/support.rs
ui-copied-review-before-submitting = Copied. Review before submitting

# Used in: root/handlers/telegram.rs
ui-confirm-the-connection-in-telegram-if-no = Confirm the connection in Telegram. If no window appears, copy the link.

# Used in: root/handlers/telegram.rs
ui-link-copied-open-it-in-telegram-saved = Link copied. Open it in Telegram Saved Messages.

# Used in: views/dns/mod.rs
ui-running-without-administrator-privileges = Running without administrator privileges

# Used in: views/dns/mod.rs
ui-dns-and-doh-changes-require-administrator-privileges = DNS and DoH changes require administrator privileges. Use Restart as administrator in Settings.

# Used in: views/dns/mod.rs
ui-dns-servers = DNS servers

# Used in: views/dns/mod.rs
ui-public-dns-helps-with-dns-based-blocking = Public DNS helps with DNS-based blocking. Proxy DNS (XBox DNS, Comss One) routes selected services through third-party servers that can see the domains you request.

# Used in: views/dns/mod.rs
ui-restore-previous-settings = Restore previous settings

# Used in: views/dns/mod.rs
ui-reset-to-dhcp-automatic = Reset to DHCP (automatic)

# Used in: views/dns/preset_card.rs
ui-applied-doh-registered = Applied · DoH registered

# Used in: views/dns/preset_card.rs
ui-applied = Applied

# Used in: views/dns/preset_card.rs
ui-reapply = Reapply

# Used in: views/dns/preset_card.rs
ui-apply = Apply

# Used in: views/overlays/whats_new.rs, views/settings/support.rs
ui-what-s-new = What's new

# Used in: views/overlays/whats_new.rs
ui-continue = Continue

# Used in: views/settings/appearance.rs
ui-system = System

# Used in: views/settings/appearance.rs
ui-light = Light

# Used in: views/settings/appearance.rs
ui-dark = Dark

# Used in: views/settings/appearance.rs
ui-appearance = Appearance

# Used in: views/settings/appearance.rs
ui-theme = Theme

# Used in: views/settings/appearance.rs
ui-interface-language = Interface language

# Used in: views/settings/automation.rs
ui-automation = Automation

# Used in: views/settings/automation.rs
ui-restart-winws-after-a-crash = Restart winws after a crash

# Used in: views/settings/automation.rs
ui-retest-strategies-when-access-is-lost = Retest strategies when access is lost

# Used in: views/settings/automation.rs
ui-retest-once-a-day = Retest once a day

# Used in: views/settings/automation.rs
ui-testing-temporarily-stops-zapret-the-app-and = Testing temporarily stops Zapret. The app and Zapret must be running with administrator privileges. Tests selected AI services. With automatic switching off, daily testing only recommends a better strategy.

# Used in: views/settings/automation.rs
ui-notifications = Notifications

# Used in: views/settings/automation.rs
ui-about-errors = About errors

# Used in: views/settings/automation.rs
ui-about-strategies-and-tests = About strategies and tests

# Used in: views/settings/automation.rs
ui-about-updates = About updates

# Used in: views/settings/automation.rs
ui-notifications-appear-when-the-window-is-minimized = Notifications appear when the window is minimized.

# Used in: views/settings/behavior.rs, views/overlays/onboarding/mod.rs
ui-startup = Startup

# Used in: views/settings/behavior.rs, views/overlays/onboarding/services.rs
ui-start-zapret-with-zui = Start Zapret with zUI

# Used in: views/settings/behavior.rs, views/overlays/onboarding/services.rs
ui-start-telegram-proxy-with-zui = Start Telegram proxy with zUI

# Used in: views/settings/behavior.rs
ui-start-at-windows-sign-in = Start at Windows sign-in

# Used in: views/settings/behavior.rs, views/overlays/onboarding/startup.rs
ui-autostart-does-not-elevate-privileges-zapret-requires = Autostart does not elevate privileges. Zapret requires an administrator launch.

# Used in: views/settings/behavior.rs
ui-application = Application

# Used in: views/settings/behavior.rs
ui-close-to-tray = Close to tray

# Used in: views/settings/behavior.rs
ui-hiding-the-window-keeps-services-running-exit = Hiding the window keeps services running; Exit stops them.

# Used in: views/settings/behavior.rs
ui-open-setup-wizard = Open setup wizard

# Used in: views/settings/behavior.rs
ui-exit-application = Exit application

# Used in: views/settings/support.rs
ui-about = About

# Used in: views/settings/support.rs
ui-check-for-updates = Check for updates

# Used in: views/settings/support.rs
ui-open-data-folder = Open data folder

# Used in: views/settings/support.rs
ui-report-a-problem = Report a problem

# Used in: views/settings/support.rs
ui-the-report-includes-the-version-checks-statistics = The report includes the version, checks, statistics and the last 250 log entries. Proxy secrets are excluded. Review the text before publishing.

# Used in: views/settings/support.rs
ui-save-report = Save report

# Used in: views/settings/support.rs
ui-copy-for-github = Copy for GitHub

# Used in: views/strategies/card.rs
ui-failed-to-start = failed to start

# Used in: views/strategies/card.rs
ui-select = Select

# Used in: views/strategies/card.rs
ui-running-2 = Running

# Used in: views/strategies/card.rs
ui-selected = Selected

# Used in: views/strategies/mod.rs
ui-choose-a-strategy-for-your-isp-changing = Choose a strategy for your ISP. Changing the active strategy automatically restarts Zapret.

# Used in: views/strategies/mod.rs
ui-tests-strategies-against-discord-and-youtube-zapret = Tests strategies against Discord and YouTube. Zapret temporarily stops and connectivity may be unstable.

# Used in: views/strategies/mod.rs
ui-dpi-bypass-strategies = DPI bypass strategies

# Used in: views/strategies/testing.rs
ui-testing-requires-administrator-privileges = Testing requires administrator privileges.

# Used in: views/strategies/testing.rs
ui-strategy-testing = Strategy testing

# Used in: views/strategies/testing.rs
ui-test-strategies = Test strategies

# Used in: views/strategies/testing.rs
ui-cancel = Cancel

# Used in: views/strategies/testing.rs
ui-select-best = Select best

# Used in: views/telegram/connection.rs
ui-downloading = Downloading…

# Used in: views/telegram/connection.rs, views/overlays/onboarding/services.rs
ui-installed = Installed

# Used in: views/telegram/connection.rs
ui-install-from-github = Install from GitHub

# Used in: views/telegram/connection.rs
ui-connection = Connection

# Used in: views/telegram/connection.rs
ui-stop = Stop

# Used in: views/telegram/connection.rs
ui-start = Start

# Used in: views/telegram/connection.rs
ui-connect-telegram = Connect Telegram

# Used in: views/telegram/connection.rs
ui-copy-link = Copy link

# Used in: views/telegram/files.rs
ui-files-and-diagnostics = Files and diagnostics

# Used in: views/telegram/files.rs
ui-open-log = Open log

# Used in: views/telegram/files.rs
ui-open-configuration = Open configuration

# Used in: views/telegram/files.rs
ui-flowseal-releases = Flowseal releases

# Used in: views/telegram/media.rs
ui-if-photos-or-videos-fail-to-load = If photos or videos fail to load

# Used in: views/telegram/media.rs
ui-use-dc-4-only = Use DC 4 only

# Used in: views/telegram/media.rs
ui-clear-dc-ip = Clear DC → IP

# Used in: views/telegram/mod.rs
ui-flowseal-proxy-runs-separately-from-zapret-connect = Flowseal proxy. Runs separately from Zapret; connect it in Telegram Desktop after starting.

# Used in: views/telegram/mod.rs
ui-the-port-status-confirms-that-the-local = The port status confirms that the local proxy is running. Check Telegram connectivity in Telegram itself.

# Used in: views/telegram/mod.rs
ui-stop-the-proxy-try-dc-4-only = Stop the proxy, try DC 4 only, then start it again. If that does not help, clear DC → IP.

# Used in: views/overlays/onboarding/language.rs
ui-choose-interface-language = Choose interface language

# Used in: views/overlays/onboarding/mod.rs
ui-language = Language

# Used in: views/overlays/onboarding/mod.rs
ui-permissions = Permissions

# Used in: views/overlays/onboarding/mod.rs
ui-services = Services

# Used in: views/overlays/onboarding/mod.rs
ui-back = Back

# Used in: views/overlays/onboarding/mod.rs
ui-finish = Finish

# Used in: views/overlays/onboarding/mod.rs
ui-next = Next

# Used in: views/overlays/onboarding/mod.rs
ui-first-time-setup = First-time setup

# Used in: views/overlays/onboarding/permissions.rs
ui-administrator-privileges-available = Administrator privileges available

# Used in: views/overlays/onboarding/permissions.rs
ui-viewing-and-telegram-proxy-work-without-elevation = Viewing and Telegram proxy work without elevation. Zapret, DNS and hosts require administrator privileges.

# Used in: views/overlays/onboarding/permissions.rs
ui-restart-as-administrator = Restart as administrator

# Used in: views/overlays/onboarding/services.rs
ui-install = Install

# Used in: views/overlays/onboarding/services.rs
ui-telegram-proxy = Telegram proxy

# Used in: views/overlays/onboarding/startup.rs
ui-start-zui-when-you-sign-in-to = Start zUI when you sign in to Windows?

# Used in: views/overlays/onboarding/startup.rs
ui-enable = Enable

# Used in: views/overlays/onboarding/startup.rs
ui-disable = Disable

# Used in: views/overlays/whats_new.rs
whats-new-summary = Favorite strategies, quick search and a clearer window-close flow — with an interface that is easier to use and translate.
whats-new-footer = You can open these notes again in Settings → About.
whats-new-notes = ## Strategies, closer at hand
    - Mark strategies with a star: favorites stay at the top.
    - Search by name or file and show favorites only.
    - See how many strategies match, then clear filters in one click.

    ## A clear choice when closing
    - Choose whether to hide zUI to the tray or exit the application.
    - Remember your choice, or bring the question back in Settings.
    - Hiding keeps services running; exiting stops them.

    ## Ready for more languages
    - The interface and release notes use Fluent language files.
    - Add a translation without editing Rust code.
    - Community language packs can be installed without rebuilding the app.

    ## More comfortable navigation
    - Lucide icons make the sidebar easier to scan.
    - Scroll over the content or the free space beside it.
    - Filters follow strategy testing; empty results explain what to do next.

# Dynamic UI messages

# Used in: tray.rs
tray-window-unavailable = Windows window is unavailable

# Used in: tray.rs
tray-registration-failed = Could not register tray messages

# Used in: tray.rs
tray-handler-failed = Could not attach tray handler

# Used in: tray.rs
tray-icon-failed = Windows could not create a tray icon

# Used in: tray.rs
tray-windows-only = The tray is only supported on Windows

# Used in: controllers/telegram.rs
telegram-installing = Downloading the official release and verifying SHA-256…

# Used in: controllers/telegram.rs
telegram-media-saved = Settings saved. Start the proxy again.

# Used in: controllers/telegram.rs
install-interrupted = Error: installation was interrupted

# Used in: controllers/services.rs
services-interrupted = Service check was interrupted

# Used in: controllers/diagnostics.rs
test-stop-failed = Strategy test cancelled: could not stop Zapret

# Used in: controllers/diagnostics.rs
test-interrupted = Strategy test was interrupted

# Used in: controllers/update.rs
update-check-interrupted = Update check was interrupted

# Used in: controllers/update.rs
worker-interrupted = Worker was interrupted

# Used in: controllers/zapret/poll.rs
install-worker-interrupted = Installation error: worker was interrupted

# Used in: controllers/zapret/process.rs
zapret-exited = The process exited. See the event log for details.

# Used in: controllers/zapret/process.rs
zapret-installing = Installing Zapret

# Used in: controllers/zapret/process.rs
zapret-testing = Testing strategies

# Used in: controllers/zapret/process.rs
zapret-no-strategies = Error: no strategies available

# Used in: controllers/zapret/process.rs
zapret-stop-failed = Could not stop the process. See the log.

# Used in: controllers/zapret/mod.rs, controllers/zapret/process.rs, controllers/zapret/strategies.rs
zapret-stopped = Stopped

# Used in: controllers/zapret/mod.rs
zapret-missing-strategies = No strategies. Check runtime/zapret.

# Used in: controllers/dns.rs
dns-applying = Configuring DNS…

# Used in: controllers/dns.rs
dns-system-settings = Original system settings

# Used in: controllers/dns.rs
dns-snapshot-present = A DNS recovery snapshot is available

# Used in: controllers/dns.rs
dns-interrupted = Error: DNS task was interrupted. The snapshot is saved on disk.

# Used in: controllers/update.rs, state/background.rs, components/cards/update_banner.rs
update-version-available = Version { $version } is available

# Used in: components/cards/update_banner.rs
update-version-installed-restart = Updated to { $version }. Restart required.

# Used in: controllers/update.rs
update-version-installed = Updated to { $version }

# Used in: controllers/update.rs, components/cards/update_banner.rs
update-error = Update error: { $error }

# Used in: components/cards/update_banner.rs, controllers/zapret/poll.rs
zapret-update-available = New Zapret version available: { $version }

# Used in: components/cards/status_card.rs, views/telegram/files.rs
data-directory = Directory: { $directory }

# Used in: views/strategies/testing.rs
test-progress = Checked { $done } of { $total }…

# Used in: views/logs.rs
logs-count = { $count ->
            [one] Total: { $count } entry
           *[other] Total: { $count } entries
        }

# Used in: views/strategies/card.rs
test-score = { $passed }/{ $total }, { $latency } ms

# Used in: root/lifecycle.rs
tray-unavailable = Tray unavailable: { $error }. Use the button in Settings to exit.

# Used in: platform/startup.rs
uac-launch-failed = UAC: launch cancelled or unavailable ({ $code })

# Used in: controllers/telegram.rs
telegram-error = TG WS Proxy error: { $error }

# Used in: controllers/telegram.rs
operation-error = Error: { $error }

# Used in: controllers/telegram.rs
telegram-link-error = Link error: { $error }

# Used in: controllers/telegram.rs
telegram-installed = TG WS Proxy { $version } installed. Click Start.

# Used in: controllers/telegram.rs, controllers/zapret/poll.rs
install-error = Installation error: { $error }

# Used in: controllers/update.rs
update-check-error = Could not check for updates: { $error }

# Used in: controllers/zapret/mod.rs
strategies-load-error = Could not load strategies: { $error }

# Used in: controllers/zapret/mod.rs
runtime-strategies = Runtime directory: { $directory } · strategies: { $count }

# Used in: controllers/zapret/process.rs
zapret-running = Running (PID: { $pid })

# Used in: controllers/zapret/process.rs
zapret-start-error = Start failed: { $error }

# Used in: controllers/zapret/strategies.rs
strategy-selected = Selected strategy: { $strategy }

# Used in: controllers/zapret/poll.rs
zapret-update-check-error = Zapret update check: { $error }

# Used in: controllers/zapret/poll.rs
zapret-installed = Zapret { $version } installed

# Used in: controllers/diagnostics.rs
test-complete-count = Strategy test complete: checked { $count }

# Used in: state/lifecycle.rs
settings-load-error = Settings error: { $error }

# Used in: state/lifecycle.rs
telegram-stop-error = Could not stop TG WS Proxy: { $error }

# Used in: controllers/dns.rs
dns-error = DNS error: { $error }

# Used in: views/settings/support.rs
version-label = Version: { $version }

# Used in: state/settings.rs
settings-save-error = Save failed: { $error }

# Used in: state/strategies.rs
strategy-changed = Strategy changed: { $strategy }

# Used in: state/background.rs
update-ready-version = Update ready. Restart the app: { $version }

# Used in: views/statistics.rs
duration-days-hours = { $days } d { $hours } h

# Used in: views/statistics.rs
duration-hours-minutes = { $hours } h { $minutes } min

# Used in: views/statistics.rs
duration-minutes = { $minutes } min

# Used in: views/statistics.rs
time-ago = { $duration } ago

# Used in: views/dns/preset_card.rs
dns-preset-public-description = Public DNS. Does not bypass DPI or geographic restrictions.

# Used in: views/dns/preset_card.rs
dns-preset-comss-description = Comss One DNS: bypassing blocks and geographic restrictions (third-party service)

# Used in: views/home.rs
telegram-proxy-label = Telegram proxy:

# Used in: views/telegram/mod.rs
telegram-title = Telegram · TG WS Proxy

# Process observations and native Windows UI

# Used in: views/settings/appearance.rs
locale-credit = Translation: { $names }

# Used in: controllers/dns.rs
dns-applied = DNS settings applied

# Used in: controllers/dns.rs
dns-restored = Original DNS settings restored

# Used in: controllers/dns.rs
dns-reset = DNS reset to DHCP

# Used in: root/view_data.rs
telegram-stopped = Stopped

# Used in: root/view_data.rs
telegram-starting = Starting…

# Used in: root/view_data.rs
telegram-listening = Listening on { $endpoint } · connect Telegram

# Used in: root/view_data.rs
telegram-exited = TG WS Proxy exited ({ $status }). See the proxy log for details.

# Used in: root/view_data.rs
telegram-process-error = TG WS Proxy process check error: { $error }

# Used in: root/view_data.rs
telegram-port-unresponsive = Process running, but the port is not responding. Open the proxy log.

# Used in: tray.rs
tray-tooltip = zUI — open application

# Used in: tray.rs
tray-open = Open zUI

# Used in: tray.rs
tray-exit = Exit

# Used in: root/lifecycle.rs
window-close-tray-failed = Could not hide to tray. Disable close-to-tray or click Exit application.

