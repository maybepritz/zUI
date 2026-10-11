# zUI

Графический интерфейс для [zapret-discord-youtube](https://github.com/Flowseal/zapret-discord-youtube) на Rust и GPUI: установка и запуск Zapret, выбор стратегии, проверка нейросервисов, Telegram-прокси и управление hosts в одном окне.

> Проект не связан с авторами Zapret и остальных перечисленных ниже программ. Они не несут ответственности за zUI.

## Возможности

- **Мастер первого запуска:** язык, права администратора, сервисы и автозапуск.
- **Установка и обновление Zapret** прямо из приложения: файлы скачиваются в папку `runtime` рядом с исполняемым файлом.
- **Telegram WS Proxy:** установка и запуск вместе с приложением.
- **Проверка нейросервисов** и автоматический подбор рабочей стратегии.
- **Watchdog:** ограниченный перезапуск `winws` после сбоя.
- **Ежедневный перетест** по желанию пользователя и статистика работы.
- **Отчёт для GitHub issue** с очисткой чувствительных строк журнала.
- Системная тема, русский и английский интерфейс, уведомления, сворачивание в трей, автообновление с GitHub.

Автоматические проверки отключены по умолчанию. Тест стратегий временно останавливает Zapret.

## Требования

- Windows 10 или 11.
- Права администратора для Zapret, DNS и hosts. Без них доступны просмотр и Telegram-прокси.

## Установка

1. Скачайте последний релиз со страницы [Releases](https://github.com/maybepritz/zui/releases).
2. Запустите `zui.exe` (для Zapret — от имени администратора).
3. Пройдите мастер первого запуска.

## Сборка

Нужны Rust (stable) и Windows.

```powershell
git clone https://github.com/maybepritz/zui.git
cd zui
cargo build --release
```

Также есть скрипт `build.ps1`.

## Структура

| Путь            | Назначение                                                      |
| --------------- | --------------------------------------------------------------- |
| `crates/app`    | Приложение: интерфейс, состояние, экраны                        |
| `crates/z-core` | Ядро: Zapret, DNS, hosts, стратегии, Telegram-прокси, настройки |
| `runtime`       | Скачиваемые Zapret и прокси (создаётся рядом с exe)             |

## Сообщить о проблеме

В приложении: **Настройки → Сообщить о проблеме**. Отчёт содержит версию, результаты проверок, статистику и последние 250 строк журнала. Секреты прокси не включаются, но проверьте текст перед публикацией. Затем откройте [issue](https://github.com/maybepritz/zui/issues/new).

## Благодарности

zUI существует благодаря работе этих людей и проектов:

- **[bol-van](https://github.com/bol-van)** — автор [zapret](https://github.com/bol-van/zapret) и `winws`, на которых всё держится.
- **[Flowseal](https://github.com/Flowseal)** — [zapret-discord-youtube](https://github.com/Flowseal/zapret-discord-youtube) (набор стратегий и сборка Zapret для Windows) и [tg-ws-proxy](https://github.com/Flowseal/tg-ws-proxy) (локальный MTProto-прокси для Telegram Desktop).
- **[peshk0v](https://github.com/peshk0v)** — [Zapret-Zen](https://github.com/peshk0v/Zapret-Zen), вдохновивший интерфейс.
- **[Zed Industries](https://zed.dev)** — [GPUI](https://gpui.rs), UI-фреймворк.
- **[Longbridge](https://longbridge.com)** — [gpui-component и GPUI Kit](https://gpui-kit.com), компоненты интерфейса.
- **[Lucide](https://lucide.dev)** — иконки.

Если вы автор проекта, который используется в zUI, но не указан здесь, откройте issue, и я добавлю.

## Лицензия

MIT
