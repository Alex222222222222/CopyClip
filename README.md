# Copy Clip

A MacOS app used to manage your clipboard history.

<!-- vscode-markdown-toc -->

1. [Install](#install)
    1.1. [MacOS](#macos)
    1.2. [Linux](#linux)
    1.3. [TODO](#todo)
2. [Known Issues](#known-issues)
    2.1. [MacOS Security Policy](#macos-security-policy)
    2.2. [Other Issues](#other-issues)
3. [Build](#build)
    3.1. [Prerequisites](#prerequisites)
    3.2. [Build](#build-the-app)
    3.3. [Run](#run)
4. [Bonnie](#bonnie)
    4.1. [Install](#install-bonnie)
    4.2. [Use](#use-bonnie)
    4.3. [Config](#config-bonnie)

<!-- vscode-markdown-toc-config
    numbering=true
    autoSave=true
    /vscode-markdown-toc-config -->

<!-- /vscode-markdown-toc -->

## Configuration

This is some explanation for configurations in the app.

| name | default value | description |
| ---- | ------------- | ----------- |
| clips per page | 20 | this define how much clips to show in one page in the tray |
| max clip length | 50 | when a clip is too long, we cut it to under max-clip-length to fit it into the tray |
| log level filter | info | the log level of the app. `trace`, `info`, `debug`, `warn`, `error`, `off` from most detailed to less |
| dark mode | off | switch the dark mode on and off |

More configurations is on the way.

## Install

### MacOS

As `sqlite3` is already installed by the system,
there is no need to install any dependency.

Just copy the app to the application folder.

### Linux

`xcb`dependency is needed, to monitor the clipboard.

Use `dpkg` to install the `deb` bundle.

## TODO

- [x] Search Page

- [x] Configuration Page

- [ ] Explanation for configurations

- [ ] i18n support

  - [ ] Chinese

- [ ] Sign the app for MacOS build

- [ ] Change the icon of the app so it can be seen in the white background

- [x] Export the history to a file

  - [ ] import the history

## Known Issues

### MacOS Security Policy

The Mac aarch64 build may have a problem with the macOS security policy,
apple need developer to buy a 99$ per year development program to
let developer to be treated as trusted developer.

You may need to manually run the following command

```bash
sudo spctl --master-disable
sudo xattr -r -d com.apple.quarantine {{the location of your app}}
```

If the problem has not been solved, use the x64 build.

### Other Issues

If you have any other issues, please open an issue with the log file attached,
and information about your system.

Also, please switch the log mod to `trace` to get the most detailed log.

The log file is located at `~/Library/Logs/org.eu.huazifan.copyclip/log`on MacOS.

## Build

### Prerequisites

You need to have the following installed:

- Rust
- pnpm
- wasm32-unknown-unknown
- tauri-cli
- trunk
- bonnie
- wasm-opt
- twiggy
- wasm-snip
- tailwindcss

```bash
# wasm32-unknown-unknown
rustup target add wasm32-unknown-unknown

# Install the Tauri CLI
cargo install tauri-cli

# Install the Trunk CLI
cargo install trunk

# Install Bonnie
cargo install bonnie

# Install wasm-opt
cargo install wasm-opt

# Install twiggy
cargo install twiggy

# Install wasm-snip
cargo install wasm-snip

# Install tailwindcss
pnpm install tailwindcss@latest
```

For linux, you need to install extra dependency: - `libxcb*`

```bash
sudo apt-get update
sudo apt install libdbus-1-dev libwebkit2gtk-4.0-dev build-essential curl wget libssl-dev libgtk-3-dev \
    libayatana-appindicator3-dev librsvg2-dev xcb libxcb-randr0-dev libxcb-xtest0-dev libxcb-xinerama0-dev \
    libxcb-shape0-dev libxcb-xkb-dev libxcb-xfixes0-dev
```

### Build The App

```bash
bonnie build frontend
```

### Run

```bash
bonnie run frontend
```

## Bonnie

Use ![Bonnie](https://github.com/arctic-hen7/bonnie) to manage the repo.

### Install Bonnie

```bash
cargo install bonnie
```

### Use Bonnie

```bash
# Build
bonnie build frontend

# Run
bonnie run frontend
```

### Config Bonnie

```toml
version="0.3.2"

[scripts]
## Builds Tailwind CSS for development (no purging)
build-tailwind-dev = [
    "tailwindcss -c ./tailwind.config.js -o ./tailwind.css"
]
## Builds Tailwind CSS for production (maximum purging and minification)
build-tailwind-prod = [
    "NODE_ENV=production tailwindcss -c ./tailwind.config.js -o ./tailwind.css --minify"
]
## Builds Tailwind CSS for development usage
setup.subcommands.tailwind = "bonnie build-tailwind-dev"
setup.subcommands.prompt-tailwind = "echo \"Have you installed the Tailwind CLI globally with 'npm i -g tailwindcss' or 'yarn global add tailwindcss'?\""
setup.order = """
tailwind {
    Failure => prompt-tailwind
}
"""

## Builds everything
build.cmd = "cargo build"
## Builds the frontend
build.subcommands.frontend = [
    "bonnie build-tailwind-prod",
    "cargo tauri build"
]
## Runs the frontend, watching for changes (uses Trunk)
## Tailwind is assumed to be set up after `setup`
run.subcommands.frontend = [
    "cargo tauri dev"
]
```

## Cargo Outdated

```bash
cargo install cargo-outdated
cargo outdated
```
