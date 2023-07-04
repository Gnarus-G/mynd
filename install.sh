set -xe;

pnpm tauri build;
cd src-tauri;
cargo build --release -p todo;
cp target/release/mynd ~/.local/bin/
cp target/release/todo ~/.local/bin/
