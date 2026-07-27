# Mynd

Mynd is a fast, private todo CLI with one installable web interface for desktop and mobile.

The CLI, language server, and web service share local files under `~/mynd`; the server listens only on loopback and can optionally be exposed to your own devices with Tailscale Serve.

## Features

- Fast `todo "message"` capture from any terminal
- Add, complete, delete, clear, and reorder from the PWA
- Installable desktop and mobile interface
- Installed desktop app detection with default-browser fallback
- Optional private HTTPS access through Tailscale
- Binary or JSON local persistence
- Todo language and LSP support
- Cross-process locking and atomic writes across CLI, LSP, and web requests

## Supported Systems

Mynd supports Linux distributions using systemd user services; the installer builds natively, so it does not assume a specific distribution, CPU architecture, hostname, browser, home-directory layout, or Tailscale network.

Required tools:

- Git
- Node.js and npm
- A stable Rust toolchain with Cargo
- systemd with a running user manager
- curl and standard POSIX utilities
- The native build tools required by Rust crates on your distribution

Tailscale is optional and is only needed for private access from other devices.

The installer runs entirely as the current user and does not use `sudo`.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/Gnarus-G/mynd/main/install.sh | sh
```

The installer:

1. Builds Mynd in a temporary directory for the current architecture.
2. Installs `todo`, `mynd`, and `mynd-server` under `~/.local/bin` by default.
3. Writes a systemd user unit under `${XDG_CONFIG_HOME:-~/.config}/systemd/user`.
4. Starts a loopback-only service on `127.0.0.1:4280` by default.
5. Offers optional Tailscale Serve configuration through `/dev/tty`, including when invoked with `curl | sh`.

No repository checkout or build directory is retained after installation.

### Installer Options

Pass environment variables to `sh`, not to `curl`:

```sh
curl -fsSL https://raw.githubusercontent.com/Gnarus-G/mynd/main/install.sh |
  MYND_HTTP_PORT=4281 MYND_TAILSCALE_PORT=8445 sh
```

| Variable | Default | Purpose |
| --- | --- | --- |
| `MYND_BIN_DIR` | `$HOME/.local/bin` | Binary installation directory |
| `MYND_HTTP_PORT` | `4280` | Loopback HTTP port |
| `MYND_TAILSCALE_PORT` | `8444` | Tailnet HTTPS port |
| `MYND_ENABLE_TAILSCALE` | prompt | Set to `1` to enable or `0` to skip noninteractively |
| `MYND_REPOSITORY` | official GitHub repository | Alternate Git remote |
| `MYND_REF` | `main` | Branch or tag to install |

Example noninteractive installation with Tailscale:

```sh
curl -fsSL https://raw.githubusercontent.com/Gnarus-G/mynd/main/install.sh |
  MYND_ENABLE_TAILSCALE=1 sh
```

If the chosen HTTP or HTTPS port is already occupied, select another with the corresponding variable; the installer never binds the Mynd server to a LAN interface.

## Use

Capture a todo:

```sh
todo "send the proposal"
```

Open the graphical app:

```sh
mynd
# or
todo gui
```

The launcher follows this order:

1. Launch an installed Mynd PWA registered through the standard XDG desktop-app directory.
2. Open the configured Mynd URL with the default browser when no installed app can be launched.

Print the configured URL:

```sh
todo url
```

Common commands:

```text
todo [MESSAGE]
todo done <ID>...
todo rm <ID>...
todo ls [--full] [--quiet]
todo dump
todo edit
todo import <FILE>
todo config show
todo config set --format binary|json
todo config set --web-url https://host.example.ts.net:8444
todo lsp
```

## Install The PWA

Open Mynd over HTTPS or localhost and use its install invitation; Chromium-based desktop browsers show a native **Install** button, while other supported browsers provide installation through their own menus.

- Chromium, Chrome, Brave, and Edge on Linux: select **Install** or **Install page as app**.
- Android browsers: select **Install app** or **Add to Home Screen**.
- iOS and iPadOS browsers: use the Share menu and select **Add to Home Screen**.
- Firefox desktop does not currently provide native manifest-based PWA installation.

After desktop installation, `mynd` prefers the registered PWA and falls back to the default browser if the registration is absent or cannot launch.

## Tailscale

Tailscale is optional; Mynd does not require a particular tailnet name or machine hostname, and the installer derives the current MagicDNS name from `tailscale status`.

Manual setup with default ports:

```sh
tailscale serve --bg --https=8444 http://127.0.0.1:4280
tailscale status
todo config set --web-url https://YOUR-CURRENT-MAGICDNS-NAME:8444
```

Use `tailscale serve status` to inspect active routes; Mynd should remain tailnet-only and should not be exposed with Tailscale Funnel.

The PWA's **Mobile** panel displays the configured URL and generates its QR code locally without contacting a third-party QR service.

## Service Operations

```sh
systemctl --user status mynd
journalctl --user -u mynd -f
systemctl --user restart mynd
systemctl --user disable --now mynd
```

The service starts with the user's systemd session; users who intentionally need it before login can enable systemd lingering according to their distribution's policy.

Default installation paths:

```text
~/.local/bin/todo
~/.local/bin/mynd
~/.local/bin/mynd-server
${XDG_CONFIG_HOME:-~/.config}/systemd/user/mynd.service
~/mynd/todo.bin
```

## Uninstall

With the default paths:

```sh
systemctl --user disable --now mynd
rm -f ~/.config/systemd/user/mynd.service
systemctl --user daemon-reload
rm -f ~/.local/bin/todo ~/.local/bin/mynd ~/.local/bin/mynd-server
tailscale serve --https=8444 off
```

The uninstall commands deliberately leave `~/mynd` intact; remove that directory separately only when its todo data is no longer needed.

## Development

Run the API and Vite in separate terminals:

```sh
npm ci
npm run build
cargo run -p mynd-server
npm run dev
```

Vite proxies `/api` to `127.0.0.1:4280`; release builds embed `build/` inside `mynd-server`.

Verify the workspace:

```sh
sh -n install.sh
npm run check
npm run build
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## LSP Setup

```lua
require("lspconfig").todols.setup({
  cmd = { "todo", "lsp" },
  filetypes = { "todolang" },
  single_file_support = true,
})
```

The todo syntax tree-sitter grammar is available at [Gnarus-G/tree-sitter-todolang](https://github.com/Gnarus-G/tree-sitter-todolang).
