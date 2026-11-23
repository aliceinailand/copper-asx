# ASX / Alison Scorpion Browser — Copper fork branding map

Shallow checkout of [copper-browser/copper-linux](https://github.com/copper-browser/copper-linux) for a future ASX rebrand.
**Do not build/replace `/usr/bin/copper` from this tree in the current goal.** Product path remains `external-tools/asx-browser/` wrapping the installed Copper package.

- Upstream HEAD at clone: `6bf5309` (Remove Search / Office Commun branding; Copper logo only)
- License: MIT (fork allowed)
- Engine: WebKitGTK via `copper-engine-webkitgtk` — keep system WebKit; never bundle Chromium

## Branding points (replace Copper → ASX / Alison Scorpion)

| Location | Current | ASX target (future) |
|----------|---------|---------------------|
| `crates/copper-app/src/lib.rs` `APP_ID` | `com.copperbrowser.Copper` | e.g. `com.alisonscorpion.ASXBrowser` |
| `crates/copper-app/src/platform.rs` window `.title("Copper")` | Copper | ASX Browser / Alison Scorpion Browser |
| `crates/copper-app/src/window/header.rs` default title `"Copper"` | Copper | ASX Browser |
| `packaging/share/com.copperbrowser.Copper.desktop` | Name=Copper, Exec=copper, Icon/WMClass=com.copperbrowser.Copper | Name=ASX Browser, GenericName=Alison Scorpion Browser, Exec=asx-browser (or renamed binary) |
| `packaging/share/com.copperbrowser.Copper.metainfo.xml` | id/name/summary Copper | ASX / Alison Scorpion metadata |
| `packaging/icons/com.copperbrowser.Copper.*` | Copper icons | ASX / Scorpion icons |
| `packaging/deb.toml` install paths | com.copperbrowser.Copper.* | matching new app id |
| `crates/copper-app` binary name `copper` | copper | asx-browser or alison-scorpion-browser |
| Creds schema `com.copperbrowser.Copper.Login` | Copper keyring schema | ASX schema (migration plan required) |

## Out of scope for thin fork-prep
- Full Rust rebuild / shipping a renamed `.deb`
- Chrome extensions (WebKitGTK has none) — use Copper **addons** instead
- Replacing Alice’s BrowserOS neo agent profile

## Product path today
- Launcher: `external-tools/asx-browser/bin/asx-browser` → `/usr/bin/copper`
- Bridge: `127.0.0.1:4124`
- Public desktop: ASX Browser primary in `alisonscorpionpublic/desktop`

## ASX Satellite UI renames (2026-11-16)
- Copper **Space** → ASX **Container** (chips, New Space, default name).
- Settings › Codecs (video MSE pack); Settings › Engine (WebKit version / Flatpak).
- Top bar: **Complexity** (auto + manual model lanes).
- New-tab icons: New tab | New files (ftools set) | New notes | + New Container.
- Troubleshoot: `docs/asx-satellite/troubleshoot/`.
