#!/bin/sh
set -eu

repository="${MYND_REPOSITORY:-https://github.com/Gnarus-G/mynd}"
revision="${MYND_REF:-main}"
bin_dir="${MYND_BIN_DIR:-$HOME/.local/bin}"
config_home="${XDG_CONFIG_HOME:-$HOME/.config}"
unit_dir="$config_home/systemd/user"
http_port="${MYND_HTTP_PORT:-4280}"
tailscale_port="${MYND_TAILSCALE_PORT:-8444}"

validate_port() {
  case "$2" in
    ''|*[!0-9]*)
      printf '%s must be a numeric TCP port.\n' "$1" >&2
      exit 1
      ;;
  esac
  if [ "$2" -lt 1 ] || [ "$2" -gt 65535 ]; then
    printf '%s must be between 1 and 65535.\n' "$1" >&2
    exit 1
  fi
}

validate_port MYND_HTTP_PORT "$http_port"
validate_port MYND_TAILSCALE_PORT "$tailscale_port"

for command in cargo curl git install mktemp node npm systemctl; do
  command -v "$command" >/dev/null 2>&1 || {
    printf 'Missing required command: %s\n' "$command" >&2
    exit 1
  }
done

case "$(uname -s)" in
  Linux) ;;
  *)
    printf 'Mynd currently supports Linux only.\n' >&2
    exit 1
    ;;
esac

systemctl --user show-environment >/dev/null 2>&1 || {
  printf 'A running systemd user manager is required.\n' >&2
  exit 1
}

work_dir=$(mktemp -d "${TMPDIR:-/tmp}/mynd-install.XXXXXX")
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM
source_dir="$work_dir/source"

printf 'Building Mynd %s for %s...\n' "$revision" "$(uname -m)"
git clone --depth 1 --branch "$revision" "$repository" "$source_dir"
npm ci --prefix "$source_dir"
npm run --prefix "$source_dir" build
cargo build --manifest-path "$source_dir/Cargo.toml" --workspace --release

mkdir -p "$bin_dir" "$unit_dir"
install -m 0755 "$source_dir/target/release/todo" "$bin_dir/.todo.new"
install -m 0755 "$source_dir/target/release/mynd" "$bin_dir/.mynd.new"
install -m 0755 "$source_dir/target/release/mynd-server" "$bin_dir/.mynd-server.new"
mv -f "$bin_dir/.todo.new" "$bin_dir/todo"
mv -f "$bin_dir/.mynd.new" "$bin_dir/mynd"
mv -f "$bin_dir/.mynd-server.new" "$bin_dir/mynd-server"

cat >"$unit_dir/mynd.service" <<EOF
[Unit]
Description=Mynd PWA server
After=network.target

[Service]
ExecStart="$bin_dir/mynd-server" --bind 127.0.0.1:$http_port
Restart=on-failure
RestartSec=2

[Install]
WantedBy=default.target
EOF

systemctl --user daemon-reload
systemctl --user enable mynd.service
systemctl --user restart mynd.service
"$bin_dir/todo" config set --web-url "http://127.0.0.1:$http_port"

attempts=0
until curl --fail --silent "http://127.0.0.1:$http_port/api/health" >/dev/null; do
  attempts=$((attempts + 1))
  if [ "$attempts" -ge 40 ]; then
    printf 'Mynd did not become ready; inspect: journalctl --user -u mynd\n' >&2
    exit 1
  fi
  sleep 0.25
done

setup_tailscale="${MYND_ENABLE_TAILSCALE:-}"
if command -v tailscale >/dev/null 2>&1 && [ -z "$setup_tailscale" ] && [ -r /dev/tty ]; then
  printf 'Expose Mynd privately through Tailscale Serve? [y/N] ' >/dev/tty
  read -r reply </dev/tty
  case "$reply" in
    y|Y|yes|YES) setup_tailscale=1 ;;
    *) setup_tailscale=0 ;;
  esac
fi

if [ "$setup_tailscale" = "1" ]; then
  command -v tailscale >/dev/null 2>&1 || {
    printf 'MYND_ENABLE_TAILSCALE=1 was set, but tailscale is not installed.\n' >&2
    exit 1
  }
  tailscale serve --bg --https="$tailscale_port" "http://127.0.0.1:$http_port"
  dns_name=$(tailscale status --json | node -e 'let input="";process.stdin.on("data",chunk=>input+=chunk);process.stdin.on("end",()=>process.stdout.write(JSON.parse(input).Self.DNSName.replace(/\.$/,"")))')
  web_url="https://$dns_name:$tailscale_port"
  "$bin_dir/todo" config set --web-url "$web_url"
  printf 'Tailnet URL: %s\n' "$web_url"
elif command -v tailscale >/dev/null 2>&1; then
  printf 'Tailscale skipped; setup instructions are in the README.\n'
fi

case ":$PATH:" in
  *":$bin_dir:"*) ;;
  *) printf 'Add %s to PATH to run todo and mynd directly.\n' "$bin_dir" ;;
esac

printf 'Mynd installed for the current user; launch it with: %s/mynd\n' "$bin_dir"
