rustup target add wasm32-unknown-unknown
rustup component add rust-src clippy rustfmt
cargo install tauri-cli
cargo install trunk --locked
sudo apt-get update
sudo apt-get install -y --no-install-recommends \
    libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev \
    libssl-dev libayatana-appindicator3-dev librsvg2-dev libdbus-1-dev \
    libgtk-3-dev xcb libxcb-randr0-dev libxcb-xtest0-dev \
    libxcb-xinerama0-dev libxcb-shape0-dev libxcb-xkb-dev libxcb-xfixes0-dev
