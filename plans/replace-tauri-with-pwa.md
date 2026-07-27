# Replace Tauri With an Installable PWA

## Decision

Replace the Tauri desktop application with one SvelteKit PWA served by a local Rust HTTP service.

- The PWA is the only graphical client on desktop and mobile.
- `mynd` becomes a lightweight launcher that opens the canonical PWA URL in the default browser.
- `todo gui` continues to work by invoking the same launcher behavior.
- A systemd user service keeps the HTTP service running on the Linux host.
- Tailscale Serve exposes that loopback-only service to the tailnet over HTTPS.
- Tailnet access is the only authentication boundary for v1.
- Todo data remains online-only; the service worker caches the application shell, not API responses or mutations.

Opening a URL cannot portably force every browser to launch its installed standalone PWA window; browsers that support link capture may do so, while other browsers will open the same application in a normal tab.

## Goals

- Preserve the fast `todo` CLI workflow and existing persisted data.
- Support add, complete, permanent delete, delete-completed, and reorder from touch and pointer devices.
- Install the same UI as a PWA on phones and desktops.
- Keep the backend unreachable from the LAN or public internet by default.
- Remove Tauri, WebKit, and native desktop bundling dependencies.
- Produce simple Linux installation, startup, update, and recovery paths.

## Non-Goals

- Offline todo reads or writes.
- A separate Mynd account, login page, session, or cloud service.
- Public access through Tailscale Funnel.
- Native mobile or desktop packages.
- Supporting non-Linux server hosts in the first iteration.
- Automatically controlling browser-specific PWA installation or window behavior.

## Target Architecture

### Rust Workspace

Move the reusable todo crate out of the Tauri tree and create a normal root Rust workspace:

```text
Cargo.toml
crates/
  todo/                 # Existing domain, persistence, CLI, LSP, and launcher bins
  mynd-server/          # Axum API and embedded static PWA
```

The `todo` package continues to expose its library and `todo` executable, and adds the small `mynd` launcher executable; `mynd-server` depends on the `todo` library and owns HTTP concerns only.

### Processes

```text
phone/desktop browser
        |
        | HTTPS inside the tailnet
        v
Tailscale Serve
        |
        | HTTP to 127.0.0.1 only
        v
mynd-server (systemd user service)
        |
        v
existing ~/mynd persistence

todo CLI and LSP --------^ via the shared Rust domain/persistence crate
```

`mynd-server` must bind to `127.0.0.1`, never `0.0.0.0`; Tailscale Serve terminates HTTPS and proxies to the loopback port, and no CORS headers are needed because the PWA and API share one origin.

### Static Application Delivery

- Keep SvelteKit with `adapter-static` and preserve client-side rendering.
- Build the frontend before `mynd-server` and embed the generated `build/` assets in the server binary.
- Serve `index.html` as the navigation fallback and serve fingerprinted assets with long-lived immutable caching.
- Serve the manifest and service worker with cache headers that permit prompt update checks.
- During development, run Vite separately and proxy `/api` to a loopback `mynd-server` process.

## API

Use same-origin JSON endpoints under `/api`:

| Method | Path | Behavior |
| --- | --- | --- |
| `GET` | `/api/health` | Service readiness |
| `GET` | `/api/todos` | Reload and return all todos |
| `POST` | `/api/todos` | Add a todo from `{ "message": string }` |
| `POST` | `/api/todos/{id}/complete` | Mark one todo done |
| `DELETE` | `/api/todos/{id}` | Permanently delete one todo |
| `DELETE` | `/api/todos/completed` | Permanently delete all completed todos |
| `POST` | `/api/todos/{id}/move-up` | Move one todo up |
| `POST` | `/api/todos/{id}/move-down` | Move one todo down |
| `POST` | `/api/todos/{id}/move-below` | Move below `{ "target_id": string }` |

Mutation responses return the updated todo collection so the existing frontend state model remains straightforward; malformed payloads return `400`, missing IDs return `404`, conflicts return `409`, and unexpected persistence failures return a generic `500` while details remain in service logs.

Only accept JSON for request bodies, do not enable CORS, and keep all GET routes free of side effects so an unrelated website cannot issue a simple cross-origin mutation through a tailnet-connected browser.

## Persistence And Concurrency

Removing Tauri still leaves the long-running server, short-lived CLI commands, and the LSP able to touch the same save file, so the current process-local `Mutex` is insufficient.

Before exposing mutations over HTTP:

1. Add characterization tests for binary and JSON persistence plus every collection mutation.
2. Add a cross-process advisory lock associated with the active save file.
3. Make each mutation one locked read-modify-write transaction rather than mutating a stale in-memory snapshot and flushing later.
4. Write replacement data to a sibling temporary file, flush it, and atomically rename it over the save file.
5. Make reads reload from storage so CLI, LSP, and HTTP changes become visible immediately.
6. Preserve `~/mynd/todo.bin`, `~/mynd/todo.json`, current config selection, IDs, timestamps, ordering, and done state without migration.

The HTTP service may retain synchronization for concurrent handlers, but correctness must come from the shared transactional persistence boundary used by every process.

## Frontend

### Data Access

- Replace direct `@tauri-apps/api` invocation with a typed same-origin `fetch` client.
- Keep API transport separate from Svelte stores and components.
- Represent loading, mutation-in-progress, offline, and server-error states explicitly.
- Refresh on initial load, window focus, and reconnection.
- Do not implement optimistic reordering initially; render the authoritative collection returned by each mutation.

### Mobile And Desktop Interaction

- Retain keyboard-efficient todo entry on desktop.
- Use minimum 44px touch targets and account for mobile safe areas.
- Keep explicit up/down controls as the reliable touch reorder mechanism.
- Preserve pointer drag-and-drop only as an enhancement, not as the sole path for reorder or delete.
- Add an explicit permanent-delete action with confirmation or undo-safe interaction instead of requiring a trash-can drop target.
- Make complete and delete-completed actions accessible by name and keyboard.
- Avoid viewport assumptions such as fixed `h-screen`; use dynamic viewport units and safe overflow regions.
- Verify narrow phones, landscape phones, tablets, and desktop widths.

### PWA

- Add `@vite-pwa/sveltekit` with a generated application-shell service worker.
- Add a manifest with Mynd name, short name, standalone display mode, theme/background colors, start URL, scope, and maskable icons.
- Establish one editable source logo, reusing the existing artwork only if it remains legible at small sizes.
- Generate and commit favicon, 192px, 512px, maskable 512px, and 180px Apple touch icon variants under `static/icons/`.
- Keep critical logo artwork inside the maskable icon safe area and verify it against light and dark platform backgrounds.
- Add manifest screenshots for phone and desktop install surfaces after the responsive UI is final.
- Exclude `/api/**` from runtime caching and show a clear online-required state when requests fail.
- Provide a small update-available prompt so an active page is not replaced during a mutation.
- Add appropriate Apple touch icon and mobile web-app metadata for iOS installation.

### Mobile Access

- Add a visible "Open on mobile" action to the desktop UI.
- Show the canonical Tailscale HTTPS URL as a selectable, copyable link.
- Render a QR code containing only that URL so a tailnet-connected phone can open it directly.
- Use the Web Share API when available, with copy-to-clipboard as the fallback.
- Include brief Android and iOS install instructions next to the link without implying that installation can be automated.
- Explain that the phone must have Tailscale connected and be authorized by the tailnet policy.
- Keep the QR code and link local to the rendered page; do not send the URL or QR payload to an external generation service.
- Hide or clearly disable the mobile-sharing action until a canonical HTTPS URL is configured.

## Launcher Behavior

Install a small `mynd` executable alongside `todo`:

1. Read the canonical web URL from Mynd config.
2. Validate that it is an `https` Tailscale URL or an explicit loopback development URL.
3. Open it through the platform default URL handler on Linux.
4. Return a useful error if the URL is unconfigured or the browser launch fails.

Change `todo gui` to call the shared launcher function instead of replacing itself with the former Tauri executable; the installation flow derives the host's MagicDNS HTTPS URL after configuring Tailscale Serve and stores it through `todo config set`, while preserving a manual `--web-url` configuration path and a command that prints the URL for sharing or troubleshooting.

## Service And Tailscale Setup

Install this user unit at `~/.config/systemd/user/mynd.service`:

```ini
[Unit]
Description=Mynd PWA server
After=network.target

[Service]
ExecStart=%h/.local/bin/mynd-server --bind 127.0.0.1:4280
Restart=on-failure
RestartSec=2

[Install]
WantedBy=default.target
```

The installer then:

1. Builds the PWA and Rust workspace in the required order.
2. Installs `todo`, `mynd`, and `mynd-server` under `~/.local/bin`.
3. Installs, enables, and starts the systemd user unit.
4. Waits for `http://127.0.0.1:4280/api/health` to become ready.
5. Runs `tailscale serve --bg http://127.0.0.1:4280` after explicit confirmation.
6. Reads the resulting HTTPS MagicDNS URL without reading or storing Tailscale credentials.
7. Stores that canonical URL in Mynd config.
8. Prints the clickable URL and PWA installation instructions.

Tailscale setup must fail safely when Tailscale is absent, disconnected, or unauthorized: the loopback service remains usable, no Funnel is configured, and the installer prints the exact recovery command instead of weakening the bind address.

Provide documented commands for service status, logs, restart, Tailscale Serve status, URL reconfiguration, and complete uninstallation; do not make the systemd unit itself invoke or continuously mutate Tailscale configuration because `tailscale serve --bg` already persists across restarts.

## Tauri Removal

Remove only after the web path is functionally complete:

- Delete the Tauri application crate, commands, capabilities, configuration, and build script.
- Remove `@tauri-apps/api`, `@tauri-apps/plugin-shell`, and `@tauri-apps/cli`.
- Remove Tauri-specific Vite host/HMR logic and Android environment helpers.
- Remove WebKit, GTK, app-indicator, Android, and Tauri build prerequisites from CI and documentation.
- Replace Tauri screenshots and usage text with PWA installation and browser-launch instructions.
- Preserve source artwork that is useful for PWA icons before deleting obsolete bundle assets.

## Build, CI, And Release

- Add root scripts for frontend checks, frontend build, Rust formatting, Rust linting, Rust tests, server build, and the complete release build.
- Update CI to install Bun and Rust, build the PWA, build the workspace, run frontend checks, run Rust tests, and test the embedded static application without native GUI packages.
- Package all three Linux binaries together rather than publishing a Tauri bundle and separate CLI archive.
- Ensure release builds fail if the frontend output is missing or stale.
- Add an API integration test that exercises the Axum router without opening a network port.
- Add an installation smoke test that starts `mynd-server` on an ephemeral loopback port and fetches the PWA and health endpoint.
- Validate the web manifest and assert that every referenced icon and screenshot is present in the built application.

## Implementation Sequence

1. Add characterization and concurrency tests around the current domain and persistence behavior.
2. Move `src-tauri/todo` into the root Rust workspace without changing behavior, and verify CLI and LSP tests.
3. Implement cross-process transactional persistence and update all CLI/LSP mutation paths to use it.
4. Add `mynd-server`, API handlers, static embedding, structured errors, and router integration tests.
5. Replace the frontend's Tauri transport with the HTTP client and verify the existing desktop UI against the server.
6. Implement touch-first controls and responsive layout while retaining desktop keyboard and drag enhancements.
7. Add the source logo, generated icon set, manifest screenshots, PWA manifest, service worker, online-required state, and update flow.
8. Add `mynd`, change `todo gui`, and extend config with the canonical web URL.
9. Add the mobile-access link, local QR code, share/copy controls, and platform installation guidance.
10. Update installation, systemd, Tailscale Serve setup, CI, release packaging, and operational documentation.
11. Verify the replacement end to end, then delete Tauri and its dependencies.

## End-To-End Verification

Verification is complete only after observing all affected flows, not merely passing checks:

- Existing binary and JSON data load unchanged after upgrading.
- `todo "message"`, `todo ls`, `todo done`, `todo rm`, editor/LSP actions, and web mutations immediately see one another's changes.
- Concurrent CLI and API mutations do not lose updates or corrupt either save format.
- Add, complete, permanent delete, delete-completed, move up/down, and move-below work in a desktop browser.
- The same flows work at a phone viewport using touch controls without drag-and-drop.
- The PWA installs and launches in standalone mode on at least one desktop Chromium browser and one phone browser.
- The displayed mobile URL opens from a tailnet-connected phone, the QR code resolves to the same URL, and no QR data reaches a third party.
- Manifest icons render correctly when installed on Android and desktop, and the Apple touch icon renders correctly on iOS.
- Reloading while offline shows the shell and an online-required state, with no stale API data presented as current.
- Restarting `mynd-server`, the systemd user service, and the host preserves data and restores access.
- `mynd` and `todo gui` open the configured canonical URL and report actionable errors when misconfigured.
- The service accepts connections on loopback, is available through Tailscale Serve HTTPS, and is not listening on LAN interfaces.
- A device outside the tailnet cannot reach the service, and `tailscale funnel status` shows no public exposure.
- Removing all Tauri/WebKit packages does not affect build, installation, CLI, LSP, launcher, or PWA operation.

## Completion Criteria

- Tauri code and dependencies no longer exist in the repository.
- One PWA provides the full graphical workflow on desktop and mobile.
- The server starts automatically as a systemd user service and is exposed only through loopback plus Tailscale Serve.
- Existing persisted data and CLI/LSP workflows remain intact.
- The launcher, installer, CI, releases, operations documentation, and end-to-end verification all cover the new architecture.
