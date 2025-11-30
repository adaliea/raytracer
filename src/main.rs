use std::error::Error;
use std::path::Path;
use clap::Parser;
use env_logger::Env;
use raytracer::{load_and_save_scene, RenderParameters};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Scene file to load (relative to the executable)
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
    
    let params = RenderParameters::new(
        args.width,
        args.height,
        args.samples,
        args.max_bounces,
    );
    
    load_and_save_scene(Path::new(&args.scene_path), params)
}
