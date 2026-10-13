# Translating zUI

Translations live in `crates/app/locales/`, using
[Project Fluent](https://projectfluent.org/fluent/guide/). Rust code contains
stable message IDs; translators own wording, sentence order and plural forms.
English is the reference catalog; Russian is also bundled.

## Add a language

1. Copy `crates/app/locales/en/` into a folder named after its language code,
   for example `de`, `pt-BR` or `zh-Hant`.
2. Edit `locale.json`. Use the native name and credit the translators:

   ```json
   {
     "name": "Deutsch",
     "translators": ["Your name or username"]
   }
   ```

3. Translate values in `main.ftl`, keeping message IDs and variable names.
   Include `whats-new-summary`, `whats-new-footer` and `whats-new-notes`,
   which contain the release overview and notes.
4. Validate from the repository root:

   ```powershell
   cargo run -p z-i18n --bin check-locales
   cargo test -p z-i18n
   ```

5. Submit the folder as a pull request. No Rust enum, registration list, manifest
   or language-picker code needs changing. The next build embeds the folder,
   which appears in Settings and the first-run wizard.

The checker does not compile GPUI or launch the application. Add `--offline`
when dependencies are available. To check an alternative directory, use:

```powershell
cargo run -p z-i18n --bin check-locales -- PATH
```

PATH must contain the English reference folder and the packs to check.
The reference is read from PATH/en.

## Install without rebuilding

Place the two files in `runtime/locales/<code>/`, then restart zUI:

```text
zUI.exe
runtime/
  locales/
    de/
      locale.json
      main.ftl
```

If zUI uses `--runtime` or `ZAPRET_UI_RUNTIME`, use that directory instead.
The picker discovers the pack automatically. Translator credits appear under
the selected language in Settings.

A local pack can translate only some messages. It overrides matching bundled
messages; missing values use the bundled version of the same language, then
English. Exact locale codes are preferred; a regional locale can use its base
language, such as ru-RU → ru. An unavailable language uses English.

Invalid metadata, Fluent syntax or duplicate IDs are rejected and logged.
Bundled catalogs remain available. Missing variables, unresolved references and
formatting errors fall back per message. Contribution validation is stricter:
a complete pack must match English IDs and effective variable sets.

## Variables and plurals

```ftl
ui-cancel = Cancel
test-progress = Checked { $done } of { $total }…
data-directory = Directory: { $directory }
```

Translate the whole value. Variables can move to any position required by the
language. Do not translate their names or add variables the app cannot supply.
Numeric arguments are numbers, enabling Fluent's plural and number formatting.

```ftl
logs-count = { $count ->
    [one] Total: { $count } entry
   *[other] Total: { $count } entries
}
```

Languages may use different categories. Russian uses one/few/many/other.
Selectors need a default branch marked `*`. Categories such as `[one]` are
grammar names, not displayed text, and must not be translated.

Multiline values use indentation. Keep release-note headings and bullets
indented inside `whats-new-notes`. Literal braces need escaping, e.g. `{ "{" }`.
Comments begin with `#` outside a message. Fluent terms (`-brand-name`) and
message references are supported. The checker follows references, verifies
variables and catches missing/circular references, including inactive branches.
NUMBER is available; custom functions and DATETIME are not registered.
Fluent's bidirectional isolation for variable values is retained.

## Developer API

```rust
use crate::i18n::{tr, tr_args};

let cancel = tr("ui-cancel");
let progress = tr_args("test-progress", &[
    ("done", done.into()),
    ("total", total.into()),
]);
```

Add new messages to both English and Russian. Keep IDs stable when changing
wording. Use full messages with named arguments rather than concatenating
translated fragments with format!.

Stored statuses use deferred Text, so a language change updates the display
without restarting the service:

```rust
self.status = crate::i18n::text_args("zapret-running", &[("pid", pid.to_string().into())]);
```

Text is formatted when converted to a label. Historical log entries are strings
and keep the language used when the event occurred. Raw errors, PowerShell
warnings and third-party executable output remain diagnostic details in their
original language; the surrounding UI is localized. Issue reports retain their
existing diagnostic format for maintainers.

The language setting remains a JSON string, including old ru/en values.
Community codes can be saved without changing the settings schema. Catalog
discovery and filesystem access happen once at startup, not during render.
Language changes refresh GPUI, native tray tooltips and menu labels.

## Review checklist

The checker validates syntax, duplicate/missing/unknown IDs, values, variables,
references and supported functions. Tests additionally cover source IDs,
fallback layering, plurals and concurrent formatting. Tests load all bundled
locale folders, including new contributions.

Review long labels at the minimum window size, the wizard, Settings, statuses,
tray menu and release notes. Automated checks verify neither visual layout nor
translation quality. Keep product names, paths and protocol terms where needed
rather than translating code identifiers.

The workspace build also produces a standalone `check-locales.exe`; it can check
a directory without installing Rust on the translator's machine:

```powershell
check-locales.exe PATH
```
