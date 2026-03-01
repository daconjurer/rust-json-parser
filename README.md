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

```bash
cargo run --bin parse_file -- path-to/file.json
```

## Python bits

To build the Python package, run any of the following:

```bash
# Development (builds and installs into your venv)
maturin develop
```

```bash
# Release build (optimized)
maturin develop --release
```

```bash
# Build a wheel without installing
maturin build --release
```

Once built, the parser tool can be run as a module like:

```bash
python3 -m rust_json_parser path-to-json/file.json
```

And a benchmark function is also exposed in the Python CLI as a flag:

```bash
python -m rust_json_parser --benchmark <path-to-dir-with-json-files>
```
