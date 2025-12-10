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


# Credits
## Textures:
https://polyhaven.com/a/laminate_floor_02
https://polyhaven.com/a/painted_plaster_wall
https://polyhaven.com/a/whitewashed_brick
https://polyhaven.com/a/rocky_terrain
https://polyhaven.com/a/dry_riverbed_rock
https://polyhaven.com/a/gravelly_sand
https://polyhaven.com/a/plank_flooring_03
https://polyhaven.com/a/cobblestone_pavement
https://polyhaven.com/a/dirty_concrete
https://polyhaven.com/a/coral_fort_wall_02