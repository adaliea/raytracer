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


# Extra Credit Work
- Denoising with OIDN (Open Image Denoise)
  - Significantly reduces the number of samples required to get pretty images
    - Even 1 sample/pixel can get nice looking results! Useful for test renders
  - Also rendering normals & albedo values to improve denoising quality
  - Can be seen as the `_albedo`, `_normal`, & `_noisy` images
- Tonemapping
  - Implemented basic tonemaping from the internal linear HDR image to the final SDR image.
  - Applies gamma correction as well
- BVH Tree utilized
  - Implemented a crate for creating BVH trees to optimize performance. 
  - Significantly speeds up rendering when there are many triangles (as a result of below)
- Normal Maps
- Displacement maps
  - Uses displacement maps to create much more detailed geometry on triangles
  - Displaced geometry is implemented as part of the loader and is created during this stage.
- Recursive render
  - A PBR raytracer creates really pretty scenes
- Next Event Estimation (NEE)
  - Trace additional rays towards light on every bounce on a lambertian surface.
  - Handled in an energy correct way
  - Similar to the idea of shadow rays from the assignment
- Lights are real objects and have volume
  - NEE rays trace to a random point on the light
  - Bounce rays can physically hit the light
  - Creates nice soft shadows on objects
- No ambient light
  - All ambient lighting happens through random bounce lighting
  - Looks much better
- Multithreaded
  - Rendering work is on a row by row basis utilizing the rayon library
  - Can use all your CPU cores.
- Performant!
  - BVH trees and optimized Rust code can calculate 11.5 million ray intersections / second in a scene with ~550k triangles
  - Simpler scenes can reach as high as 200 million ray intersections / second (with BVH trees disabled)
- Easy to use command line interface
- Rust
  - All original memory safe code written by me in Rust

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