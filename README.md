# Copy Clip

A MacOS app used to manage your clipboard history.

<!-- vscode-markdown-toc -->

* 1. [Install](#Install)
     * 1.1. [MacOS](#MacOS)
     * 1.2. [Linux](#Linux)
     * 1.3. [TODO](#TODO)
* 2. [Known Issues](#KnownIssues)
     * 2.1. [MacOS Security Policy](#MacOSSecurityPolicy)
     * 2.2. [Other Issues](#OtherIssues)
* 3. [Build](#Build)
     * 3.1. [Prerequisites](#Prerequisites)
     * 3.2. [Build](#Build-1)
     * 3.3. [Run](#Run)
* 4. [Bonnie](#Bonnie)
     * 4.1. [Install](#Install-1)
     * 4.2. [Use](#Use)
     * 4.3. [Config](#Config)

<!-- vscode-markdown-toc-config
    numbering=true
    autoSave=true
    /vscode-markdown-toc-config -->

<!-- /vscode-markdown-toc --># Copy Clip

## Install

### MacOS

As `sqlite3` is already installed by the system, there is no need to install any dependency.

Just copy the app to the application folder.

### Linux

`xcb`dependency is needed, to monitor the clipboard.

Use `dpkg` to install the `deb` bundle.

### TODO

- [x] Search Page

- [x] Configuration Page

- [ ] Explanation for configurations

- [ ] i18n support
  
  - [ ] Chinese

- [ ] Sign the app for MacOS build

- [ ] Change the icon of the app so it can be seen in the white background

## Known Issues

### MacOS Security Policy

The Mac aarch64 build may have a problem with the macOS security policy, you may need to manually run the following command

```bash
sudo spctl --master-disable
sudo xattr -r -d com.apple.quarantine {{the location of your app}}
```

If the problem has not been solved, use the x64 build.

### Other Issues

If you have any other issues, please open an issue with the log file attached, and information about your system.

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

For linux, you need to install extra dependency:
    - `libxcb*`

```bash
sudo apt-get update
sudo apt install libdbus-1-dev libwebkit2gtk-4.0-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev xcb libxcb-randr0-dev libxcb-xtest0-dev libxcb-xinerama0-dev libxcb-shape0-dev libxcb-xkb-dev libxcb-xfixes0-dev
```

### Build

```bash
bonnie build frontend
```

### Run

```bash
bonnie run frontend
```

## Bonnie

Use Bonnie to manage the repo.

https://github.com/arctic-hen7/bonnie

### Install

```bash
cargo install bonnie
```

### Use

```bash
# Build
bonnie build frontend

# Run
bonnie run frontend
```

### Config

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