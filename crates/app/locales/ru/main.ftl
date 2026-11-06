# zUI UI messages. Keep message IDs and variable names unchanged.

# Window-close dialog
close-dialog-title = Закрыть zUI?
close-dialog-hint = Скрытие оставляет zUI и сервисы работать в трее. Выход останавливает Zapret и Telegram Proxy и завершает приложение.
close-hide = Скрыть в трей
close-exit = Выйти из приложения
close-remember = Запомнить мой выбор
close-remember-setting = Закрывать окно без вопроса, используя сохранённый выбор
close-error-save = Не удалось запомнить выбор. Проверьте журнал и повторите попытку.

# Used in: views/strategies/filters.rs, root/strategy_filters.rs
strategy-list-title = Доступные стратегии
strategy-search-label = Поиск стратегий
strategy-search-placeholder = Поиск по названию или файлу…
strategy-filter-favorites = Только избранное
strategy-filter-reset = Сбросить фильтры
strategy-filter-count = Показано { $shown } из { $total }
strategy-list-empty = Нет доступных стратегий
strategy-list-empty-hint = Установите Zapret или проверьте его файлы на главной
strategy-open-home = Открыть главную
strategy-favorites-empty = Нет избранных стратегий
strategy-favorites-empty-hint = Отметьте стратегии звёздочкой или сбросьте фильтры
strategy-search-empty = Ничего не найдено
strategy-search-empty-hint = Попробуйте другой запрос или сбросьте фильтры

# Used in: views/strategies/card.rs
favorite-add = Добавить в избранное
favorite-remove = Убрать из избранного

# Used in: notifications.rs
ui-zui-error = zUI — ошибка

# Used in: notifications.rs
ui-zui-strategy = zUI — стратегия

# Used in: notifications.rs
ui-zui-update = zUI — обновление

# Used in: root/navigation.rs
ui-home = Главная

# Used in: root/navigation.rs
ui-strategies = Стратегии

# Used in: root/navigation.rs, views/statistics/
ui-statistics = Статистика

# Used in: root/navigation.rs
ui-logs = Логи

# Used in: root/navigation.rs, views/settings/mod.rs
ui-settings = Настройки

# Used in: state/background.rs
ui-winws-exited-and-was-restarted = winws завершился и был перезапущен

# Used in: state/background.rs
ui-could-not-restart-winws-automatic-retries-are = Не удалось перезапустить winws. Автоповторы ограничены тремя попытками

# Used in: state/reports.rs
ui-report-saved-review-it-before-publishing = Отчёт сохранён. Проверьте текст перед публикацией

# Used in: state/settings.rs
ui-settings-saved = Настройки сохранены

# Used in: state/testing.rs
ui-could-not-start-the-selected-strategy-the = Не удалось запустить выбранную стратегию; восстановлена предыдущая

# Used in: state/testing.rs
ui-could-not-restore-zapret-after-testing-see = Не удалось восстановить Zapret после теста. См. журнал

# Used in: state/testing.rs
ui-a-better-strategy-was-found-and-applied = Найдена и применена лучшая стратегия

# Used in: state/testing.rs
ui-a-better-strategy-is-available-on-the = Найдена лучшая стратегия. Выберите её на вкладке «Стратегии»

# Used in: state/testing.rs
ui-test-complete-current-strategy-retained = Тест завершён. Текущая стратегия сохранена

# Used in: state/testing.rs
ui-test-cancelled-previous-state-restored = Тест отменён. Предыдущее состояние восстановлено

# Used in: views/logs.rs
ui-no-events-yet-start-zapret-on-the = Лог пуст. Запустите службу Zapret на вкладке «Главная» для просмотра событий.

# Used in: views/logs.rs
ui-event-log = Журнал событий

# Used in: views/logs.rs
ui-clear-log = Очистить журнал

# Used in: i18n/time.rs
ui-just-now = только что

# Used in: views/statistics/
ui-zui-uptime = Общее время работы

# Used in: views/statistics/
ui-strategy-changes = Смен стратегии

# Used in: views/statistics/
ui-winws-restarts = Автоперезапуски

# Used in: components/cards/install_card.rs
ui-zapret-is-not-installed = Zapret не установлен

# Used in: components/cards/install_card.rs
ui-the-latest-zapret-discord-youtube-release-will = Будет скачан последний релиз zapret-discord-youtube в папку runtime рядом с программой.

# Used in: components/cards/install_card.rs
ui-installing = Установка…

# Used in: components/cards/install_card.rs
ui-install-zapret = Установить Zapret

# Used in: components/cards/update_banner.rs
ui-later = Позже

# Used in: components/cards/update_banner.rs
ui-update = Обновить

# Used in: components/cards/update_banner.rs
ui-installing-update = Установка обновления…

# Used in: components/cards/update_banner.rs
ui-restart = Перезапустить

# Used in: components/cards/update_banner.rs
ui-close = Закрыть

# Used in: components/cards/update_banner.rs
ui-updating-zapret = Обновление Zapret…

# Used in: root/handlers/support.rs
ui-report-saved = Отчёт сохранён

# Used in: root/handlers/support.rs
ui-could-not-save-the-report = Не удалось сохранить отчёт

# Used in: root/handlers/support.rs
ui-copied-review-before-submitting = Текст скопирован. Проверьте его перед отправкой

# Used in: root/handlers/telegram.rs
ui-confirm-the-connection-in-telegram-if-no = Подтвердите подключение в Telegram. Если окно не открылось, скопируйте ссылку.

# Used in: root/handlers/telegram.rs
ui-link-copied-open-it-in-telegram-saved = Ссылка скопирована. Откройте её в «Избранном» Telegram.

# Used in: views/dns/mod.rs
ui-running-without-administrator-privileges = Запущено без прав администратора

# Used in: views/dns/mod.rs
ui-dns-and-doh-changes-require-administrator-privileges = Для применения DNS и шифрования DoH требуются права Администратора. Перезапустите zapret-ui от имени Администратора.

# Used in: views/dns/mod.rs
ui-dns-servers = DNS-серверы

# Used in: views/dns/mod.rs
ui-public-dns-helps-with-dns-based-blocking = Публичные DNS помогают, только если блокировка сделана подменой DNS-ответов. Прокси-DNS (XBox DNS, Comss One) перенаправляют трафик ряда сервисов через сторонние серверы: они видят, какие домены вы запрашиваете.

# Used in: views/dns/mod.rs
ui-restore-previous-settings = Вернуть как было

# Used in: views/dns/mod.rs
ui-reset-to-dhcp-automatic = Сбросить на DHCP (Авто)

# Used in: views/dns/preset_card.rs
ui-applied-doh-registered = Применён · DoH зарегистрирован

# Used in: views/dns/preset_card.rs
ui-applied = Применён

# Used in: views/dns/preset_card.rs
ui-reapply = Переприменить

# Used in: views/dns/preset_card.rs
ui-apply = Применить

# Used in: views/overlays/whats_new.rs, views/settings/support.rs
ui-what-s-new = Что нового

# Used in: views/overlays/whats_new.rs
ui-continue = Продолжить

# Used in: views/settings/appearance.rs
ui-system = Как в системе

# Used in: views/settings/appearance.rs
ui-light = Светлая

# Used in: views/settings/appearance.rs
ui-dark = Тёмная

# Used in: views/settings/appearance.rs
ui-appearance = Внешний вид

# Used in: views/settings/appearance.rs
ui-theme = Тема

# Used in: views/settings/appearance.rs
ui-interface-language = Язык интерфейса

# Used in: views/settings/automation.rs
ui-automation = Автоматизация

# Used in: views/settings/automation.rs
ui-restart-winws-after-a-crash = Перезапускать winws после сбоя

# Used in: views/settings/automation.rs
ui-retest-strategies-when-access-is-lost = Подбирать стратегию при потере доступа

# Used in: views/settings/automation.rs
ui-retest-once-a-day = Перетест раз в сутки

# Used in: views/settings/automation.rs
ui-testing-temporarily-stops-zapret-the-app-and = Тест временно останавливает Zapret. Нужны работающее приложение, активный Zapret и права администратора. Проверяются выбранные нейросервисы. Без автопереключения ежедневный тест только предлагает лучшую стратегию.

# Used in: views/settings/automation.rs
ui-notifications = Уведомления

# Used in: views/settings/automation.rs
ui-about-errors = Об ошибках

# Used in: views/settings/automation.rs
ui-about-strategies-and-tests = О стратегиях и тестах

# Used in: views/settings/automation.rs
ui-about-updates = Об обновлениях

# Used in: views/settings/automation.rs
ui-notifications-appear-when-the-window-is-minimized = Уведомления приходят при свёрнутом окне

# Used in: views/settings/behavior.rs, views/overlays/onboarding/mod.rs
ui-startup = Запуск

# Used in: views/settings/behavior.rs, views/overlays/onboarding/components.rs
ui-start-zapret-with-zui = Запускать Zapret вместе с zUI

# Used in: views/settings/behavior.rs, views/overlays/onboarding/components.rs
ui-start-telegram-proxy-with-zui = Запускать Telegram-прокси вместе с zUI

# Used in: views/settings/behavior.rs
ui-start-at-windows-sign-in = Автозапуск при входе в Windows

# Used in: views/settings/behavior.rs, views/overlays/onboarding/startup.rs
ui-autostart-does-not-elevate-privileges-zapret-requires = Автозапуск не повышает права. Для Zapret понадобится запуск от администратора.

# Used in: views/settings/behavior.rs
ui-application = Приложение

# Used in: views/settings/behavior.rs
ui-close-to-tray = Сворачивать в трей при закрытии

# Used in: views/settings/behavior.rs
ui-hiding-the-window-keeps-services-running-exit = Скрытое окно сохраняет работу сервисов; «Выйти» останавливает их.

# Used in: views/settings/behavior.rs
ui-open-setup-wizard = Открыть мастер настройки

# Used in: views/settings/behavior.rs
ui-exit-application = Выйти из приложения

# Used in: views/settings/support.rs
ui-about = О приложении

# Used in: views/settings/support.rs
ui-check-for-updates = Проверить обновления

# Used in: views/settings/support.rs
ui-open-data-folder = Открыть папку данных

# Used in: views/settings/support.rs
ui-report-a-problem = Сообщить о проблеме

# Used in: views/settings/support.rs
ui-the-report-includes-the-version-checks-statistics = Отчёт содержит версию, результаты проверок, статистику и последние 250 строк журнала. Секреты прокси не включаются. Проверьте текст перед публикацией.

# Used in: views/settings/support.rs
ui-save-report = Сохранить отчёт

# Used in: views/settings/support.rs
ui-copy-for-github = Копировать для GitHub

# Used in: views/strategies/card.rs
ui-failed-to-start = не запустилась

# Used in: views/strategies/card.rs
ui-select = Выбрать

# Used in: views/strategies/card.rs
strategy-running = Работает

# Used in: views/strategies/card.rs
ui-selected = Выбрана

# Used in: views/strategies/mod.rs
ui-choose-a-strategy-for-your-isp-changing = Выберите наиболее подходящую стратегию под вашего провайдера. При смене активной стратегии работающий Zapret перезапустится автоматически.

# Used in: views/strategies/mod.rs
ui-tests-strategies-against-discord-and-youtube-zapret = Перебор стратегий с проверкой Discord и YouTube. Интернет на время теста нестабилен, Zapret будет остановлен.

# Used in: views/strategies/mod.rs
ui-dpi-bypass-strategies = Стратегии обхода блокировок DPI

# Used in: views/strategies/testing.rs
ui-testing-requires-administrator-privileges = Для теста нужны права администратора

# Used in: views/strategies/testing.rs
ui-strategy-testing = Подбор стратегии

# Used in: views/strategies/testing.rs
ui-test-strategies = Проверить стратегии

# Used in: views/strategies/testing.rs
ui-cancel = Отмена

# Used in: views/strategies/testing.rs
ui-select-best = Выбрать лучшую

# Used in: views/telegram/connection.rs
ui-downloading = Загрузка…

# Used in: views/telegram/connection.rs, views/overlays/onboarding/components.rs
ui-installed = Установлен

# Used in: views/telegram/connection.rs
ui-install-from-github = Установить с GitHub

# Used in: views/telegram/connection.rs
ui-connection = Подключение

# Used in: views/telegram/connection.rs
ui-stop = Остановить

# Used in: views/telegram/connection.rs
ui-start = Запустить

# Used in: views/telegram/connection.rs
ui-connect-telegram = Подключить Telegram

# Used in: views/telegram/connection.rs
ui-copy-link = Копировать ссылку

# Used in: views/telegram/files.rs
ui-files-and-diagnostics = Файлы и диагностика

# Used in: views/telegram/files.rs
ui-open-log = Открыть журнал

# Used in: views/telegram/files.rs
ui-open-configuration = Открыть конфигурацию

# Used in: views/telegram/files.rs
ui-flowseal-releases = Релизы Flowseal

# Used in: views/telegram/media.rs
ui-if-photos-or-videos-fail-to-load = Если фото и видео не загружаются

# Used in: views/telegram/media.rs
ui-use-dc-4-only = Оставить только DC 4

# Used in: views/telegram/media.rs
ui-clear-dc-ip = Очистить DC → IP

# Used in: views/telegram/mod.rs
ui-flowseal-proxy-runs-separately-from-zapret-connect = Официальный прокси Flowseal. Работает отдельно от Zapret; после запуска подключите его в Telegram Desktop.

# Used in: views/telegram/mod.rs
ui-the-port-status-confirms-that-the-local = Статус порта подтверждает запуск локального прокси. Доступность Telegram проверяется в самом Telegram.

# Used in: views/telegram/mod.rs
ui-stop-the-proxy-try-dc-4-only = Остановите прокси, попробуйте только DC 4 и запустите снова. Если не помогло — очистите DC → IP.

# Used in: views/overlays/onboarding/language.rs
ui-choose-interface-language = Выберите язык интерфейса

# Used in: views/overlays/onboarding/mod.rs
ui-language = Язык

# Used in: views/overlays/onboarding/mod.rs
ui-permissions = Права

# Used in: views/overlays/onboarding/mod.rs
ui-services = Сервисы

# Used in: views/overlays/onboarding/mod.rs
ui-back = Назад

# Used in: views/overlays/onboarding/mod.rs
ui-finish = Готово

# Used in: views/overlays/onboarding/mod.rs
ui-next = Далее

# Used in: views/overlays/onboarding/mod.rs
ui-first-time-setup = Первый запуск

# Used in: views/overlays/onboarding/permissions.rs
ui-administrator-privileges-available = Права администратора есть

# Used in: views/overlays/onboarding/permissions.rs
ui-viewing-and-telegram-proxy-work-without-elevation = Без повышения прав доступны просмотр и Telegram-прокси. Zapret и DNS требуют прав администратора.

# Used in: views/overlays/onboarding/permissions.rs
ui-restart-as-administrator = Перезапустить от администратора

# Used in: views/overlays/onboarding/components.rs
ui-install = Установить

# Used in: views/overlays/onboarding/components.rs
ui-telegram-proxy = Telegram-прокси

# Used in: views/overlays/onboarding/startup.rs
ui-start-zui-when-you-sign-in-to = Запускать zUI при входе в Windows?

# Used in: views/overlays/onboarding/startup.rs
ui-enable = Включить

# Used in: views/overlays/onboarding/startup.rs
ui-disable = Выключить

# Used in: views/overlays/whats_new.rs
whats-new-summary = Избранные стратегии, быстрый поиск и понятное закрытие окна — с интерфейсом, который удобнее использовать и переводить.
whats-new-footer = Эти заметки можно снова открыть в Настройках → О программе
whats-new-notes = ## Стратегии под рукой
    - Отмечайте стратегии звёздочкой: избранное всегда в начале списка.
    - Ищите по названию или файлу и показывайте только избранное.
    - Смотрите число найденных стратегий и сбрасывайте фильтры одним нажатием.

    ## Понятное закрытие окна
    - Выбирайте, скрыть zUI в трей или выйти из приложения.
    - Запоминайте выбор или возвращайте вопрос через настройки.
    - При скрытии сервисы продолжают работать; при выходе останавливаются.

    ## Больше возможностей для перевода
    - Интерфейс и заметки релиза вынесены в языковые файлы Fluent.
    - Добавляйте переводы без изменений Rust-кода.
    - Пользовательские языковые пакеты устанавливаются без пересборки приложения.

    ## Удобнее перемещаться по приложению
    - Lucide-иконки помогают быстрее находить разделы в боковом меню.
    - Прокручивайте страницу над содержимым и свободным местом по бокам.
    - Фильтры расположены после тестов; пустой список подсказывает следующий шаг.

# Dynamic UI messages

# Used in: tray.rs
tray-window-unavailable = Окно Windows недоступно

# Used in: tray.rs
tray-registration-failed = Не удалось зарегистрировать сообщения трея

# Used in: tray.rs
tray-handler-failed = Не удалось подключить обработчик трея

# Used in: tray.rs
tray-icon-failed = Windows не удалось создать значок в трее

# Used in: tray.rs
tray-windows-only = Трей поддерживается только в Windows

# Used in: controllers/telegram.rs
telegram-installing = Загрузка официального релиза и проверка SHA-256…

# Used in: controllers/telegram.rs
telegram-media-saved = Настройка сохранена. Запустите прокси снова.

# Used in: controllers/telegram.rs
install-interrupted = Ошибка: установка прервана

# Used in: controllers/services.rs
services-interrupted = Проверка сервисов прервана

# Used in: controllers/diagnostics.rs
test-stop-failed = Тест стратегий отменён: не удалось остановить Zapret

# Used in: controllers/diagnostics.rs
test-interrupted = Тест стратегий прерван

# Used in: controllers/update.rs
update-check-interrupted = Проверка обновлений прервана

# Used in: controllers/update.rs
worker-interrupted = поток прерван

# Used in: controllers/zapret/poll.rs
install-worker-interrupted = Ошибка установки: поток прерван

# Used in: controllers/zapret/process.rs
zapret-exited = Процесс завершился. Причина — в журнале событий.

# Used in: controllers/zapret/process.rs
zapret-installing = Идёт установка Zapret

# Used in: controllers/zapret/process.rs
zapret-testing = Идёт тест стратегий

# Used in: controllers/zapret/process.rs
zapret-no-strategies = Ошибка: нет доступных стратегий

# Used in: controllers/zapret/process.rs
zapret-stop-failed = Не удалось остановить процесс. См. журнал.

# Used in: controllers/zapret/mod.rs, controllers/zapret/process.rs, controllers/zapret/strategies.rs
zapret-stopped = Остановлено

# Used in: controllers/zapret/mod.rs
zapret-missing-strategies = Нет стратегий. Проверьте runtime/zapret.

# Used in: controllers/dns.rs
dns-applying = Выполняется настройка DNS…

# Used in: controllers/dns.rs
dns-system-settings = Исходные настройки системы

# Used in: controllers/dns.rs
dns-snapshot-present = Есть сохранённый снимок DNS

# Used in: controllers/dns.rs
dns-interrupted = Ошибка: задача DNS прервана. Снимок сохранён на диске.

# Used in: controllers/update.rs, state/background.rs, components/cards/update_banner.rs
update-version-available = Доступна версия { $version }

# Used in: components/cards/update_banner.rs
update-version-installed-restart = Обновлено до { $version }. Нужен перезапуск.

# Used in: controllers/update.rs
update-version-installed = Обновлено до { $version }

# Used in: controllers/update.rs, components/cards/update_banner.rs
update-error = Ошибка обновления: { $error }

# Used in: components/cards/update_banner.rs, controllers/zapret/poll.rs
zapret-update-available = Доступна новая версия Zapret: { $version }

# Used in: views/telegram/files.rs
data-directory = Каталог: { $directory }

# Used in: views/strategies/testing.rs
test-progress = Проверено { $done } из { $total }…

# Used in: views/logs.rs
logs-count = { $count ->
        [one] Всего: { $count } запись
        [few] Всего: { $count } записи
        [many] Всего: { $count } записей
       *[other] Всего: { $count } записей
    }

# Used in: views/strategies/card.rs
test-score = { $passed }/{ $total }, { $latency } мс

# Used in: root/lifecycle.rs
tray-unavailable = Трей недоступен: { $error }. Для выхода используйте кнопку в настройках.

# Used in: platform/autostart.rs
uac-launch-failed = UAC: запуск отменён или недоступен ({ $code })

# Used in: controllers/telegram.rs
telegram-error = Ошибка TG WS Proxy: { $error }

# Used in: controllers/telegram.rs
operation-error = Ошибка: { $error }

# Used in: controllers/telegram.rs
telegram-link-error = Ошибка ссылки: { $error }

# Used in: controllers/telegram.rs
telegram-installed = Установлен TG WS Proxy { $version }. Нажмите «Запустить».

# Used in: controllers/telegram.rs, controllers/zapret/poll.rs
install-error = Ошибка установки: { $error }

# Used in: controllers/update.rs
update-check-error = Не удалось проверить обновления: { $error }

# Used in: controllers/zapret/mod.rs
strategies-load-error = Ошибка загрузки стратегий: { $error }

# Used in: controllers/zapret/mod.rs
runtime-strategies = Каталог runtime: { $directory } · стратегий: { $count }

# Used in: controllers/zapret/process.rs
zapret-running = Работает (PID: { $pid })

# Used in: controllers/zapret/process.rs
zapret-start-error = Ошибка запуска: { $error }

# Used in: controllers/zapret/strategies.rs
strategy-selected = Выбрана стратегия: { $strategy }

# Used in: controllers/zapret/poll.rs
zapret-update-check-error = Проверка обновления Zapret: { $error }

# Used in: controllers/zapret/poll.rs
zapret-installed = Zapret { $version } установлен

# Used in: controllers/diagnostics.rs
test-complete-count = Тест стратегий завершён: проверено { $count }

# Used in: state/lifecycle.rs
settings-restored-from-backup = Не удалось прочитать настройки ({ $error }). Восстановлена предыдущая копия, подробности в журнале.

# Used in: state/lifecycle.rs
settings-reset-to-defaults = Не удалось прочитать настройки ({ $error }). Используются настройки по умолчанию, подробности в журнале.

# Used in: state/lifecycle.rs
telegram-stop-error = Ошибка остановки TG WS Proxy: { $error }

# Used in: controllers/dns.rs
dns-error = Ошибка DNS: { $error }

# Used in: views/settings/support.rs
version-label = Версия: { $version }

# Used in: state/settings.rs
settings-save-error = Ошибка сохранения: { $error }

# Used in: state/strategies.rs
strategy-changed = Стратегия изменена: { $strategy }

# Used in: state/background.rs
update-ready-version = Обновление готово. Перезапустите приложение: { $version }

# Used in: i18n/time.rs
duration-days-hours = { $days } д { $hours } ч

# Used in: i18n/time.rs
duration-hours-minutes = { $hours } ч { $minutes } мин

# Used in: i18n/time.rs
duration-minutes = { $minutes } мин

# Used in: i18n/time.rs
time-ago = { $duration } назад

# Used in: views/dns/preset_card.rs
dns-preset-public-description = Публичный DNS. Не обходит DPI и геоблок

# Used in: views/dns/preset_card.rs
dns-preset-comss-description = Comss One DNS: обход блокировок и геоблока (сторонний сервис)

# Used in: views/telegram/mod.rs
telegram-title = Telegram · TG WS Proxy

# Process observations and native Windows UI

# Used in: views/settings/appearance.rs
locale-credit = Перевод: { $names }

# Used in: controllers/dns.rs
dns-applied = Настройки DNS применены

# Used in: controllers/dns.rs
dns-restored = Исходные настройки DNS восстановлены

# Used in: controllers/dns.rs
dns-reset = DNS сброшен к DHCP

# Used in: root/view_data.rs
telegram-stopped = Остановлен

# Used in: root/view_data.rs
telegram-starting = Запускается…

# Used in: root/view_data.rs
telegram-listening = Слушает { $endpoint } · подключите Telegram

# Used in: root/view_data.rs
telegram-exited = TG WS Proxy завершился ({ $status }). Подробности в журнале прокси.

# Used in: root/view_data.rs
telegram-process-error = Ошибка проверки TG WS Proxy: { $error }

# Used in: root/view_data.rs
telegram-port-unresponsive = Процесс запущен, но порт не отвечает. Откройте журнал прокси.

# Used in: tray.rs
tray-tooltip = zUI — открыть приложение

# Used in: tray.rs
tray-open = Открыть zUI

# Used in: tray.rs
tray-exit = Выход

# Used in: root/lifecycle.rs
window-close-tray-failed = Не удалось свернуть в трей. Для выхода выключите сворачивание или нажмите «Выйти из приложения».

# Statistics: saved results and recent activity.
statistics-intro = Последняя проверка, общее время работы и недавние события
statistics-last-check = Последняя проверка
statistics-open-testing = К тестированию
statistics-check-success = Все цели доступны
statistics-check-partial = Часть целей недоступна
statistics-check-failed = Рабочая стратегия не найдена
statistics-check-cancelled = Проверка прервана
statistics-check-meta = { $time } · Проверено стратегий: { $count }
statistics-best-strategy = Лучший результат проверки
statistics-best-so-far = Лучший результат до остановки
statistics-targets = Доступно целей: { $passed } из { $total }
statistics-latency = Средняя задержка: { $ms } мс
statistics-snapshot-hint = Результат на момент проверки. Текущее состояние соединения может отличаться.
statistics-check-empty = Пока нет результатов
statistics-check-empty-hint = Запустите тестирование стратегий, чтобы сравнить доступность целей и задержку.
statistics-legacy-check = Последний успешный тест: { $time }. Подробные результаты не сохранены.
statistics-uptime-hint = За все запуски приложения
statistics-changes-hint = Выбор другой стратегии
statistics-restarts-hint = Восстановления после остановки winws
statistics-history = Последние события
statistics-history-hint = До 20 последних событий · история сохраняется между запусками
statistics-history-empty = Здесь появится история
statistics-history-empty-hint = Смены стратегии, результаты тестирования и автоматические восстановления.
statistics-event-strategy = Выбрана стратегия: { $strategy }
statistics-event-test = Проверка стратегий завершена
statistics-event-cancelled = Проверка стратегий прервана
statistics-event-recovered = winws автоматически перезапущен
statistics-event-recovery-failed = Не удалось автоматически перезапустить winws
statistics-event-service-down = { $service } не открывался { $duration }

# Startup: real preparation stages shown before the main interface.
startup-settings = Загружаем настройки…
startup-languages = Подключаем переводы…
startup-components = Проверяем компоненты…
startup-updates = Проверяем обновления…
startup-network-hint = При медленной сети проверка обновлений продолжится в фоне
startup-failed = Не удалось подготовить приложение
startup-failed-hint = Повторите запуск подготовки. Если ошибка повторится, перезапустите zUI.
startup-retry = Повторить

# Used in: views/home.rs, root/sidebar.rs
ui-advanced = Для опытных
home-bypass-on = Обход работает
home-bypass-off = Обход выключен
home-bypass-off-hint = YouTube и Discord могут не открываться
home-bypass-failed = Обход не работает
home-bypass-switch = Обход блокировок
home-service-down = { $service } не открывается
home-service-down-hint = Провайдер мог сменить блокировку
home-checking = Проверяем сервисы
home-checked = Проверено { $ago }
home-latency-day = Задержка за сутки
home-chart-empty = График появится через 15 минут после включения обхода
home-no-data = нет данных
home-axis-day-ago = сутки назад
home-axis-12h = 12 ч назад
home-axis-now = сейчас
home-service-no-outages = без сбоев за сутки
home-service-outages = не открывался { $duration } за сутки
home-service-not-responding = не отвечает
home-service-not-checked = не проверяется
home-latency-ms = { $ms } мс
home-strategy = Способ обхода: { $strategy }
home-strategy-none = не выбран
home-change = Сменить
home-check-now = Проверить сейчас
home-retest = Подобрать заново
