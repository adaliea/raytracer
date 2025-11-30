use rayon::iter::ParallelIterator;
mod camera;
mod hittable;
mod loader;
mod material;
mod ray;
mod scene;
mod tracer;

use crate::scene::Scene;
use crate::tracer::ray_color;
use glam::Vec3A;
use image::{ImageBuffer, ImageResult, Rgb};
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use rand::Rng;
use rayon::prelude::*;
use std::error::Error;
use std::fs::create_dir;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

#[derive(Debug, Copy, Clone)]

pub struct RenderParameters {
    image_width: u32,
    image_height: u32,
    samples_per_pixel: u32,
    max_bounces: u32,
    aspect_ratio: f32,
}

impl RenderParameters {
    pub fn new(
        image_width: u32,
        image_height: u32,
        samples_per_pixel: u32,
        max_bounces: u32,
    ) -> Self {
        let aspect_ratio = image_width as f32 / image_height as f32;
        Self {
            image_width,
            image_height,
            samples_per_pixel,
            max_bounces,
            aspect_ratio,
        }
    }
}
pub fn render(params: RenderParameters, scene: Scene) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    // Image
    let time = Instant::now();
    info!(
        "Rendering image {}x{}...",
        params.image_width, params.image_height
    );

    // Progress bar
    let bar = ProgressBar::new(params.image_height as u64 * params.image_width as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {per_sec} (ETA: {eta}) {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
    );

    // Render
    let mut buffer: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::new(params.image_width, params.image_height);

    let total_rays = AtomicU64::new(0);
    // parallelizes per image row
    buffer
        .enumerate_pixels_mut()
        .par_bridge()
        .for_each(|(x, y, pixel)| {
            let mut rng = rand::rng();
            let mut color = Vec3A::ZERO;
            let mut rays = 0;
            for _ in 0..params.samples_per_pixel {
                let u = (x as f32 + rng.random::<f32>() - 0.5) / (params.image_width - 1) as f32;
                let v = ((params.image_height - y - 1) as f32 + rng.random::<f32>() - 0.5)
                    / (params.image_height - 1) as f32;

                let r = scene.camera.get_ray(u, v);

                let trace = ray_color(
                    &r,
                    &scene,
                    params.max_bounces,
                    params.max_bounces,
                    true,
                    &mut rng,
                );
                color += trace.color;
                rays += trace.rays;
            }
            color /= params.samples_per_pixel as f32;

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
            total_rays.fetch_add(rays, Ordering::Release);
        });

    bar.finish();
    let elapsed = time.elapsed();
    let num_pixels = params.image_width as u64 * params.image_height as u64;
    let rays = total_rays.load(Ordering::Acquire);
    info!("Rendering finished. Traced {} rays in {} seconds. {} rays/pixels, {} rays/s", rays, &elapsed.as_secs(), rays / num_pixels, rays / elapsed.as_secs());
    buffer
}

pub fn save_buffer(buffer: ImageBuffer<Rgb<u8>, Vec<u8>>, scene_path: &Path) -> ImageResult<()> {
    let path = Path::new(scene_path);
    let filename = path.file_stem().unwrap().to_str().unwrap();
    let dir = Path::new("output");
    if !dir.exists() || !dir.is_dir() {
        create_dir(dir)
            .unwrap_or_else(|_| panic!("Failed to create output directory {}", dir.display()))
    }
    // Save image
    buffer.save(format!("output/{}.png", filename))
}

pub fn load_scene(path: &Path, aspect_ratio: f32) -> Result<Scene, Box<dyn Error>> {
    info!("Loading scene from {:?}", path);

    // Scene
    let scene = loader::load_scene(path, aspect_ratio)?;
    info!("Loaded scene with {} objects", scene.objects.len());
    Ok(scene)
}

pub fn load_and_save_scene(path: &Path, params: RenderParameters) -> Result<(), Box<dyn Error>> {
    let scene = load_scene(path, params.aspect_ratio)?;
    let rendered = render(params, scene);
    save_buffer(rendered, path).map_err(Box::from)
}
