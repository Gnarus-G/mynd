set -xe;

OPT_DIR=~/.local/opt/mynd/
BIN_DIR=~/.local/bin/

rm -rf $OPT_DIR
mkdir -p $OPT_DIR

cd $OPT_DIR

git clone https://github.com/Gnarus-G/mynd .

bun install;

NO_STRIP=true bun tauri build;
cargo build --manifest-path=./src-tauri/Cargo.toml --release -p todo;

cd src-tauri;

cp target/x86_64-unknown-linux-gnu/release/mynd $BIN_DIR
cp target/release/todo $BIN_DIR
