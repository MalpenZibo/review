# Project Setup

## Installing Rust

To install Rust, follow the [official instructions](https://www.rust-lang.org/tools/install).

Rust can compile source codes for different "targets" (e.g. different processors). The compilation target for browser-based WebAssembly is called "wasm32-unknown-unknown". The following command will add this target to your development environment.

``` bash
rustup target add wasm32-unknown-unknown
```

## Install Trunk

Trunk is a great tool for managing deployment and packaging, and will be the default choice for reView. It will be used in the documentation, in every example and in the default review-template.

``` bash
# note that this might take a while to install, because it compiles everything from scratch
# Trunk also provides prebuilt binaries for a number of major package managers
# See https://trunkrs.dev/#install for further details
cargo install trunk wasm-bindgen-cli
```

## Summary
Now that you have all the tools needed, we can build a sample application.