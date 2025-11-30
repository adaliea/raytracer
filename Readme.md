# A Raytracer Written in Rust

A multithread raytracer written in Rust.

## Usage

Some libraries use `code-simd` which requires nightly rust. You may need to use `rustup override set nightly` to use
this project.

### Running (cargo)

#### Install dependencies

We need to install `open image denoise` for image processing.

https://github.com/RenderKit/oidn/releases/download/v2.3.3/oidn-2.3.3.x86_64.linux.tar.gz`

```
cargo run --release
```

To see the help menu:

```
cargo run --release -- --help
```

### Running built binary

Located in `target/release/raytracer` or `target/x86_64-pc-windows-gnu/release/raytracer.exe'

Use `--help` to see the help menu.