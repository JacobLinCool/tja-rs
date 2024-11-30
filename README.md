# tja-rs

TJA file parser written in Rust, working in Rust, Python, and WebAssembly.

## Building

### Rust

Rust target requires no additional feature flags.

To build the library, run:

```sh
cargo build
```

To build the CLI tool, run:

```sh
cargo build --bin tja
```

### Python

We use maturin to build the Python package.

The Python package requires the `python` feature flag to be enabled.

To build the Python package, run:

```sh
maturin develop -F python
```

### WebAssembly

We use wasm-pack to build the WebAssembly package.

The WebAssembly package requires the `wasm` feature flag to be enabled.

To build the WebAssembly package, run:

```sh
wasm-pack build --features wasm
```
