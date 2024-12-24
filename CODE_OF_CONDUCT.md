# Contributor Code of Conduct

Any contribution is welcome.
However, please follow the following rules:

## General Rules

Open pull request for any change.

If you are fixing an existing issue or implementing a feature request,
please refer to the issue number in the pull request.

Please write a clear description for your pull request,
so that we can understand your change easily.

Signing off your commit is not required,
but it is recommended.

## Linting and Formatting

Run the following test before opening a pull request.
GitHub Actions will automatically run the following test for you
if you open a pull request,
but if any test failed, we will not merge your pull request.
```bash
# cargo check
cargo check
# cargo clippy for frontend
cargo clippy --all-targets --all-features -- -D warnings
# cargo clippy for backend
cargo clippy --all-targets --all-features --manifest-path src-tauri/Cargo.toml -- -D warnings
# cargo clippy for clip struct
cargo clippy --all-targets --all-features --manifest-path src-clip/Cargo.toml -- -D warnings
# cargo clippy for logging
cargo clippy --all-targets --all-features --manifest-path tauri-plugin-logging/Cargo.toml -- -D warnings
# cargo fmt
cargo fmt --all -- --check
# try to build the app
cargo tauri build
```

## Documentation

Please write documentation for your code.

If your `async` function tries to lock a mutex,
please state it in the documentation.