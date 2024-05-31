set -xe;

pnpm tauri build;
cd src-tauri;
cargo build --release -p todo;
sudo cp target/release/mynd target/release/todo ~/.local/bin/
