mod camera;
mod hittable;
mod loader;
mod material;
mod ray;
mod scene;
mod tracer;

use crate::tracer::ray_color;
use clap::Parser;
use env_logger::Env;
use glam::Vec3A;
use image::{ImageBuffer, Rgb};
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use rand::Rng;
use rayon::prelude::*;
use std::fs::create_dir;
use std::path::Path;

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

fn main() {
    color_backtrace::install();
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let args = Args::parse();

    // Image
    let aspect_ratio = args.width as f32 / args.height as f32;
    let image_width = args.width;
    let image_height = args.height;
    let samples_per_pixel = args.samples;

    info!(
        "Loading scene from {} with {} samples per pixel",
        args.scene_path, args.samples
    );
    info!("Rendering image {}x{}...", image_width, image_height);

    // Scene
    let scene = loader::load_scene(&args.scene_path, aspect_ratio)
        .expect(&format!("Failed to load scene from {}", args.scene_path));
    info!("Loaded scene with {} objects", scene.objects.len());

    // Progress bar
    let bar = ProgressBar::new(image_height as u64 * image_width as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {per_sec} (ETA: {eta}) {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
    );

    // Render
    let mut buffer: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(image_width, image_height);

    // parallelizes per image row
    buffer
        .enumerate_pixels_mut()
        .par_bridge()
        .for_each(|(x, y, pixel)| {
            let mut color = Vec3A::ZERO;
            for _ in 0..samples_per_pixel {
                let u = (x as f32 + rand::rng().random::<f32>() - 0.5) / (image_width - 1) as f32;
                let v = ((image_height - y - 1) as f32 + rand::rng().random::<f32>() - 0.5)
                    / (image_height - 1) as f32;

                let r = scene.camera.get_ray(u, v);

                color += ray_color(&r, &scene, args.max_bounces, args.max_bounces, true);
            }
            color /= samples_per_pixel as f32;

            // Basic tone mapping
            color = color / (color + Vec3A::splat(1.0));

            // Gamma correction
            color = color.powf(1.0 / 2.2);

            *pixel = Rgb([
                (color.x.clamp(0.0, 0.999) * 256.0) as u8,
                (color.y.clamp(0.0, 0.999) * 256.0) as u8,
                (color.z.clamp(0.0, 0.999) * 256.0) as u8,
            ]);
            bar.inc(1);
        });

    bar.finish();

    let path = Path::new(&args.scene_path);
    let filename = path.file_stem().unwrap().to_str().unwrap();
    let dir = Path::new("output");
    if !dir.exists() || !dir.is_dir() {
        create_dir(dir).expect(&format!(
            "Failed to create output directory {}",
            dir.display()
        ));
    }
    // Save image
    buffer.save(format!("output/{}.png", filename)).unwrap();
}
