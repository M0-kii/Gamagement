# Gamagement 🎮

Game center management with a Tauri 2 desktop shell, a Rust backend, and SQLite.
The Persian, RTL interface keeps console management, live session timers,
pause/resume, controller pricing, shop items, and checkout.

## Requirements

- Node.js 22 or newer and pnpm
- Stable Rust installed through rustup
- On Windows: Microsoft C++ Build Tools (Desktop development with C++) and WebView2
- For other platforms, install the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

## Development

```sh
pnpm install
pnpm start
```

The frontend uses plain HTML, CSS, and JavaScript. A small Node script copies
the interface and assets into `dist/`; no frontend framework is required.
Restart `pnpm start` after changing frontend sources to refresh this copy.

## Production

```sh
pnpm dist
```

On Windows this builds the NSIS installer in
`src-tauri/target/release/bundle/nsis/`. The standalone executable is
`src-tauri/target/release/gamagement.exe` and requires WebView2.
`pnpm build` builds the configured bundle. The default bundle target is Windows
NSIS; when building on macOS or Linux, choose an appropriate target with
`pnpm tauri build --bundles app` or `pnpm tauri build --bundles deb`.

## Data migration

The SQLite schema and existing pricing rules are preserved: the first two
controllers are included, extra controllers are charged per hour, paused time
is excluded, and shop purchases are added at checkout. Changing controller
counts or console rates applies the new rate to the session, as in the
previous version.

The database is stored in the data subdirectory of Tauri's local app data directory. On Windows this is
`%LOCALAPPDATA%/dev.m0-kii.gamagement/data/gamagement.db`.

On the first launch, if that database does not exist, Gamagement checks the
previous Electron location:
`%APPDATA%/dev.m0-kii.gamagement/gamagement.db` on Windows,
`~/Library/Application Support/dev.m0-kii.gamagement/gamagement.db` on macOS, or
`$XDG_CONFIG_HOME/dev.m0-kii.gamagement/gamagement.db` (defaulting to
`~/.config/`) on Linux. It copies the database using SQLite's backup API,
checks the copy, and leaves the original untouched. Close the old Electron app
before the first launch. An existing Tauri database is never overwritten.
Back up the database before moving installations between machines.

## Verification

```sh
pnpm test
pnpm test:frontend
pnpm build:frontend
```

Rust tests cover billing, pause/resume, checkout persistence, validation,
shop edits, and legacy database migration. Frontend tests check IPC argument
mapping and packaged asset references.

## Structure

```text
assets/                  Application icon and local fonts
src/                     Persian HTML/CSS UI, renderer, Tauri API bridge
scripts/                 Static frontend packaging
tests/                   Frontend integration checks
src-tauri/src/           Rust commands and SQLite business logic
src-tauri/               Tauri configuration, capabilities, Cargo dependencies
```

Developed by [M0_kii](https://github.com/M0-kii).
