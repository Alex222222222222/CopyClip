FROM mcr.microsoft.com/vscode/devcontainers/base:bullseye

# Update default packages
RUN apt update && export DEBIAN_FRONTEND=noninteractive

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

# Update new packages
RUN apt-get update

# Install Tailwind CSS
RUN curl -s https://github.com/tailwindlabs/tailwindcss/releases/download/v3.4.3/tailwindcss-linux-x64 -o /usr/local/bin/tailwindcss \
    && chmod +x /usr/local/bin/tailwindcss \
    && echo 'export PATH="$PATH:/usr/local/bin"' >> $HOME/.bashrc

# Get Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc

# Create new user and update ~/.bashrc
RUN useradd -ms /bin/bash vscode
USER vscode
WORKDIR /home/vscode
RUN echo 'export PATH="$PATH:/usr/local/bin"' >> $HOME/.bashrc \
    && echo '. "$HOME/.cargo/env"' >> $HOME/.bashrc \
    && echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
