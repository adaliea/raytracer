mod camera;
mod hittable;
mod material;
mod ray;
mod scene;
mod loader;

use std::fs::create_dir;
use std::io::Error;
use std::path::Path;
use clap::Parser;
use env_logger::Env;
use glam::Vec3A;
use image::{ImageBuffer, Rgb};
use indicatif::{ProgressBar, ProgressStyle};
use log::{info};
use rayon::prelude::*;
use crate::hittable::Hittable;
use crate::ray::Ray;
use crate::scene::Scene;

#[inline]
fn ray_color(r: &Ray, world: &Scene) -> Vec3A {
    if let Some(rec) = world.hit(r, 0.0, f32::INFINITY) {
        return 0.5 * (rec.normal + Vec3A::ONE);
    }
    let unit_direction = r.direction.normalize();
    let t = 0.5 * (unit_direction.y + 1.0);
    (1.0 - t) * Vec3A::ONE + t * Vec3A::new(0.5, 0.7, 1.0)
}


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
    #[arg(short, long, default_value_t = 1)]
    samples: u32
}

fn main() {
    let args = Args::parse();

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Image
    let aspect_ratio = args.width as f32 / args.height as f32;
    let image_width = args.width;
    let image_height = args.height;
    let samples_per_pixel = args.samples;

    info!("Loading scene from {} with {} samples per pixel", args.scene_path, args.samples);
    info!("Rendering image {}x{}...", image_width, image_height);

    // Scene
    let scene = loader::load_scene(&args.scene_path, aspect_ratio).expect(&format!("Failed to load scene from {}", args.scene_path));
    info!("Loaded scene with {} objects", scene.objects.len());


    // Progress bar
    let bar = ProgressBar::new(image_height as u64 * image_width as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    // Render
    let mut buffer: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::new(image_width, image_height);

    // parallelizes per image row
    buffer
        .enumerate_pixels_mut()
        .par_bridge()
        .for_each(|(x, y, pixel)| {
            let u = x as f32 / (image_width - 1) as f32;
            let v = (image_height - y - 1) as f32 / (image_height - 1) as f32;
            let r = scene.camera.get_ray(u, v);
            let mut color = Vec3A::ZERO;
            for _ in 0..samples_per_pixel {
                color += ray_color(&r, &scene);
            }
            color /= samples_per_pixel as f32;

            *pixel = Rgb([
                (color.x * 255.999) as u8,
                (color.y * 255.999) as u8,
                (color.z * 255.999) as u8,
            ]);
            bar.inc(1);
        });


    bar.finish();

    let path = Path::new(&args.scene_path);
    let filename = path.file_stem().unwrap().to_str().unwrap();
    let dir = Path::new("output");
    if !dir.exists() || !dir.is_dir() {
        create_dir(dir).expect(&format!("Failed to create output directory {}", dir.display()));
    }
    // Save image
    buffer.save(format!("output/{}.png", filename)).unwrap();
}