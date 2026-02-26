A Rust JSON parser (with Python bindings)
=========================================

# Getting started

To setup the development environment...

1. Install Rust

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Restart terminal and verify installation
cargo --version
rustc --version

# You should see output similar to:
cargo 1.90.0 (Homebrew)

# and:
rustc 1.90.0 (1159e78c4 2025-09-14) (Homebrew)
```

And configure the `rust-analyzer` extension in your IDE.

To use the library as a CLI tool to parse a file within the project directory, run:

```
cargo run --bin parse_file -- path-to/file.json
```

To build the Python package, run any of the following:

```
# Development (builds and installs into your venv)
maturin develop
```

```
# Release build (optimized)
maturin develop --release
```

```
# Build a wheel without installing
maturin build --release
```

