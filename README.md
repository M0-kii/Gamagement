# Gamagement

Desktop management for game centers, built with Tauri 2, Rust, and SQLite.
The interface is Persian and right-to-left.

## Features

- Console dashboard with available, playing, paused, and checkout states.
- One new session per physical console.
- Session rates fixed at start; controller changes apply to subsequent play.
- Paused time excluded from billing, with the original start timestamp retained.
- Checkout freezes the bill. Confirming payment saves the displayed amount;
  cancelling restores the previous session state.
- Session purchases entered with a name and price.
- Rounded desktop interface with remembered light/dark appearance and collapsible sidebar.
- Searchable receipts, Jalali date filters, daily revenue, console usage, and CSV export.
- Printable receipts and archived consoles with retained history.

## Run

Install Node.js 22 or newer, pnpm, and stable Rust. Windows also requires
Microsoft C++ Build Tools with the Desktop development with C++ workload and
WebView2. See the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)
for platform setup.

```sh
pnpm install
pnpm start
```

The frontend uses HTML, CSS, and JavaScript. Restart the development app after
changing frontend files; `pnpm build:frontend` copies them to `dist/`.

## Build

```sh
pnpm dist
```

The Windows installer is written to `src-tauri/target/release/bundle/nsis/`.
The executable is `src-tauri/target/release/gamagement.exe`; it requires WebView2.
For macOS or Linux, select a platform bundle with
`pnpm tauri build --bundles app` or `pnpm tauri build --bundles deb`.

## Billing

Prices and saved totals use integer tomans. Play charges are accumulated across
controller segments, then rounded once to the nearest toman; halves round up.
The first two controllers are included in the hourly price.

Console name and rates are saved with each session. Purchase names and prices
are saved directly on that session and retained in its receipt.

Opening checkout stops the timer and prevents session edits. Payment saves the
frozen bill. Returning to play excludes the checkout interval and restores the
previous active or paused state. Pending checkouts survive an app restart.

Purchases need only a name and price. They do not create shared products or
require stock management. Purchases from earlier versions remain in receipts.

## History and reports

Date filters accept Jalali dates, including Persian digits, such as
`۱۴۰۵/۰۷/۱۷`. The end date includes the entire selected day. Reports use payment
time in the Asia/Tehran timezone and include only settled sessions.

CSV export includes every matching receipt, independently of the current page.
Amounts are in tomans and exported timestamps are UTC. Receipts can be reopened
and printed from the history screen.

Clear history permanently removes all paid sessions, receipt items, and play
segments after confirmation, regardless of the current filters. Open sessions
and console definitions are retained.

## Data and upgrades

On Windows the database is located at:

```text
%LOCALAPPDATA%/dev.m0-kii.gamagement/data/gamagement.db
```

On first launch, the app can copy an earlier Electron database from
`%APPDATA%/dev.m0-kii.gamagement/gamagement.db`. The corresponding locations are
`~/Library/Application Support/dev.m0-kii.gamagement/gamagement.db` on macOS and
`$XDG_CONFIG_HOME/dev.m0-kii.gamagement/gamagement.db` (defaulting to
`~/.config/`) on Linux. The source is left untouched.

An existing database is upgraded transactionally. Before upgrading an older
schema, the app creates `gamagement.pre-v2.db` beside the database. Close the old
application before upgrading and keep a separate backup of operational data.

Legacy receipts retain their saved totals, rounded to whole tomans. Older
versions did not retain original pause-adjusted start times or controller
change intervals; these cannot be reconstructed. Migrated open sessions begin
segment tracking using the stored start time, controller count, and current
console rates. Existing duplicate sessions are kept so they can be settled;
new duplicate sessions are blocked.

## Development checks

```sh
pnpm test
pnpm test:frontend
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

The tests cover billing, checkout, stock changes, reporting, Jalali dates,
command wiring, and migration rollback.

```text
src/                  Interface, API bridge, and date formatting
assets/               Icon and Vazirmatn fonts
scripts/              Frontend packaging
tests/                Frontend regression tests
src-tauri/src/        Desktop commands, database, migrations, and reports
```

The bundled Vazirmatn fonts are distributed under the
[SIL Open Font License](assets/fonts/OFL.txt).

Maintained by [M0_kii](https://github.com/M0-kii).
