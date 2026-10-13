# Результат модульного рефакторинга

Структура согласована с сохранением root/handlers и реализована.
Подробное дерево и правила расширения находятся в [architecture.md](architecture.md).

## Что перенесено

| До | После |
| --- | --- |
| root.rs | root/mod.rs, navigation.rs, lifecycle.rs, render.rs, sidebar.rs |
| Callbacks с мутациями внутри views/components | root/handlers/{navigation,strategies,dns,telegram,settings,onboarding,support}.rs |
| Проверки процессов/файлов и чтение mutex в render | root/view_data.rs, обновление до render |
| state.rs и state/actions.rs | state/mod.rs и сценарии lifecycle/settings/strategies/startup/events/reports/updates |
| controllers/zapret.rs | controllers/zapret/{mod,process,strategies,install,poll}.rs |
| views/strategies.rs | views/strategies/{mod,card,testing}.rs |
| views/telegram.rs | views/telegram/{mod,connection,media,files}.rs |
| views/dns.rs | views/dns/{mod,preset_card}.rs |
| overlays/onboarding.rs | overlays/onboarding/{mod,language,permissions,services,startup}.rs |
| Повторяющиеся контейнеры карточек стратегии и DNS | components/cards/list_card.rs, используется обоими экранами |
| service_row внутри onboarding | components/rows/service_row.rs, используется для двух сервисов |
| stat_card внутри statistics | components/cards/stat_card.rs, используется для четырёх показателей |
| z-core/zapret/zapret.rs | zapret/{model,process,strategies}.rs |
| z-core/zapret/diagnostics.rs | diagnostics/{mod,model,runner,probe}.rs |
| z-core/telegram.rs | telegram/{mod,process,config}.rs |

Уже выделенные state/background.rs, state/testing.rs, DNS и небольшие
контроллеры сохраняют свои границы. Для UI используется общий ViewData;
отдельная папка presentation с моделью каждого экрана не вводилась.
Дополнительные crates, entity для каждой карточки и универсальные traits
для сервисов или экранов не нужны для текущего разделения.

## Типы и фасады

RootView владеет private state/Tab/OnboardingStep/Tray/ViewData.
GPUI trait Render реализован в root/render.rs, AssetSource для EmbeddedAssets
остаётся в main.rs. Именованные handlers — impl RootView, доступные в crate.
Сценарии — impl AppState в соответствующих файлах; они не принимают GPUI Context.

root/mod.rs реэкспортирует Tab, OnboardingStep и число шагов для разметки.
state/mod.rs реэкспортирует Preference для boolean-переключателей.
В библиотеке сохраняются прежние публичные пути:

```rust
use z_core::zapret::{Strategy, Zapret, load_strategies};
use z_core::zapret::diagnostics::{Options, Progress, Score, run};
use z_core::telegram::TelegramProxy;
```

Внутренние process/config/runner/probe-модули private. Соседние части
используют pub(super) для необходимых служебных функций.

## Сохранение поведения

Перенос сохраняет алгоритмы диагностики, парсинг BAT, управление процессами,
скачивание, проверку SHA-256, восстановление DNS и сценарии выбора стратегии.
Сохранены формат настроек, пути runtime, таймеры, тексты и ID существующих
элементов, порядок shutdown и обработка ошибок сохранения.

Handlers заменяют прежние callback-тела. Обновление snapshot происходит сразу
после события и на секундном tick. Проверки внешнего состояния, которые раньше
выполнялись при render, теперь выполняются до построения элементов.
Это делает render свободным от операций над процессами, файлами и mutex,
сохраняя обновление интерфейса после пользовательских действий.

Существовавшие до переноса изменения рабочей копии сохранены. Экран hosts,
удалённый до этой работы, не восстанавливался. Критерий успеха HTTP-проверки
и отсутствие проверки жизни winws внутри диагностики не менялись: исправление
этих ранее найденных проблем требует отдельного изменения поведения.

## Выполненные проверки

- cargo check --workspace --all-targets --offline: проходит.
- cargo build --workspace --offline: проходит, исполняемый файл собран.
- cargo test -p z-core --offline вне песочницы: 33 passed, 0 failed.
- cargo doc -p z-core --no-deps --offline: Rustdoc сгенерирован.
- Форматирование изменённых Rust-файлов и ссылки документации проверены.
- При переносе сопоставлены тела 72 существующих функций ядра, контроллера
  Zapret и сценариев AppState с исходной версией; различия только в
  форматировании, комментариях и видимости служебных функций.
- Сопоставлены ID элементов экранов стратегий, Telegram, DNS и onboarding.
- Проверено отсутствие прямых мутаций RootView и проверок процессов/файлов
  в views/components.

Проверка чтения DNS внутри песочницы получает отказ в доступе; полный набор
тестов прошёл при повторном запуске вне неё. После подключения общих компонентов
проверка всех targets повторно прошла; остаётся прежнее предупреждение page_fixed.
Ручной прогон Windows GUI
не выполнялся.

## Ручной прогон перед релизом

1. Первый запуск и повторное открытие onboarding; язык, UAC и запуск сервисов.
2. Переключение стратегии при работающем Zapret и отмена диагностики.
3. Настройки темы, автозапуска и уведомлений; перезапуск с сохранённым runtime.
4. Telegram: запуск/остановка, clipboard, ссылки, media и открытие файлов.
5. DNS: применение и восстановление на тестовой машине.
6. Закрытие в трей, возврат окна и выход с остановкой принадлежащих процессов.
7. Обновления, отчёт и подтверждение журнала изменений при ошибке записи.
