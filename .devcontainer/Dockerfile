FROM mcr.microsoft.com/vscode/devcontainers/base:bullseye

# Update default packages
RUN apt update \
    && apt upgrade -y \
    && export DEBIAN_FRONTEND=noninteractive

RUN apt install -y \
    build-essential \
    curl \
    wget \
    unzip \
    git \
    sudo

# Install build dependencies
RUN apt install -y libdbus-1-dev libwebkit2gtk-4.0-dev build-essential \
    curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev \
    librsvg2-dev xcb libxcb-randr0-dev libxcb-xtest0-dev libxcb-xinerama0-dev \
    libxcb-shape0-dev libxcb-xkb-dev libxcb-xfixes0-dev

# Install Tailwind CSS
RUN apt-get update \
    && apt install nodejs npm -y \
    && npm install -D tailwindcss

# Create new user and update ~/.bashrc
USER vscode
WORKDIR /home/vscode

# Get Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y \
    && echo 'export PATH="$PATH:/usr/local/bin"' >> $HOME/.bashrc \
    && echo '. "$HOME/.cargo/env"' >> $HOME/.bashrc \
    && echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
RUN $HOME/.cargo/bin/rustup target add wasm32-unknown-unknown \
    && $HOME/.cargo/bin/rustup component add rust-src clippy rustfmt \
    && $HOME/.cargo/bin/cargo install tauri-cli trunk wasm-opt
