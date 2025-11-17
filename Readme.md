# A Raytracer Written in Rust

A multithread raytracer written in Rust.

## Usage

Some libraries use `code-simd` which requires nightly rust. You may need to use `rustup override set nightly` to use
this project.

### Running (cargo)

```
cargo run --release
```

To see the help menu:

```
cargo run --release -- --help
```

### Running built binary

Located in `target/release/raytracer`

Use `--help` to see the help menu.