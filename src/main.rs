use clap::Parser;
use env_logger::Env;
use nalgebra::SimdBool;
use raytracer::{RenderParameters, load_and_save_scene};
use std::error::Error;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Scene file to load (relative to the executable)
    /// Input a directory to batch render multiple scenes
    #[arg(long, default_value = "scenes/singleSphereLighted.ray")]
    scene_path: String,

    /// Width of the image to generate
    #[arg(long, default_value_t = 400)]
    width: u32,

    /// Height of the image to generate
    #[arg(long, default_value_t = 400)]
    height: u32,

    /// Samples per pixel
    #[arg(short, long, default_value_t = 2000)]
    samples: u32,

    #[arg(short, long, default_value_t = 30)]
    max_bounces: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    color_backtrace::install();
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let args = Args::parse();

    let params = RenderParameters::new(args.width, args.height, args.samples, args.max_bounces);

    let path = Path::new(&args.scene_path);

    if path.is_dir() {
        // Render all the .ray files in the dir;

        let files = path
            .read_dir()?
            .filter(|d| {
                d.as_ref().is_ok_and(|d| {
                    d.file_name()
                        .into_string()
                        .unwrap_or_default()
                        .ends_with(".ray")
                })
            })
            .collect::<Vec<_>>();

        Ok(for scene_path in files {
            load_and_save_scene(&*scene_path?.path(), params)?
        })
    } else {
        load_and_save_scene(path, params)
    }
}
