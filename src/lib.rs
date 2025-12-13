#![feature(unboxed_closures)]

use rayon::iter::ParallelIterator;
mod camera;
mod hittable;
mod loader;
mod material;
mod ray;
mod scene;
mod tracer;

use crate::loader::SceneIterator;
use crate::scene::Scene;
use crate::tracer::ray_color;
use glam::Vec3A;
use image::{ImageBuffer, ImageResult, Rgb};
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
#[cfg(not(feature = "denoise"))]
use log::warn;
use rand::Rng;
use rayon::prelude::*;
use std::error::Error;
use std::fs::create_dir;
use std::iter::Peekable;
use std::iter::once;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
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
pub fn render_hdr(params: RenderParameters, scene: Scene) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
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
    let mut hdr_buffer = vec![Vec3A::ZERO; (params.image_width * params.image_height) as usize];
    let mut albedo_buffer = vec![Vec3A::ZERO; (params.image_width * params.image_height) as usize];
    let mut normal_buffer = vec![Vec3A::ZERO; (params.image_width * params.image_height) as usize];

    let total_rays = AtomicU64::new(0);
    // parallelizes per image row
    hdr_buffer
        .par_iter_mut()
        .zip(albedo_buffer.par_iter_mut())
        .zip(normal_buffer.par_iter_mut())
        .enumerate()
        .for_each(|(i, ((hdr_pixel, albedo_pixel), normal_pixel))| {
            let x = i as u32 % params.image_width;
            let y = i as u32 / params.image_width;

            let mut rng = rand::rng();
            let mut total_color = Vec3A::ZERO;
            let mut total_albedo = Vec3A::ZERO;
            let mut total_normal = Vec3A::ZERO;
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
                total_color += trace.color;
                total_albedo += trace.albedo;
                total_normal += trace.normal;
                rays += trace.rays;
            }
            *hdr_pixel = total_color / params.samples_per_pixel as f32;
            *albedo_pixel = total_albedo / params.samples_per_pixel as f32;
            *normal_pixel = total_normal / params.samples_per_pixel as f32;

            bar.inc(1);
            total_rays.fetch_add(rays, Ordering::Relaxed);
        });

    bar.finish();
    let elapsed = time.elapsed();
    let num_pixels = params.image_width as u64 * params.image_height as u64;
    let rays = total_rays.load(Ordering::Acquire);
    info!(
        "Rendering finished. Traced {} rays in {} seconds. {} rays/pixels, {} rays/s",
        rays,
        &elapsed.as_secs_f64(),
        rays / num_pixels,
        rays as f64 / elapsed.as_secs_f64()
    );

    // Flatten the buffer of Vec3A to a Vec<f32> for OIDN
    let flat_hdr = hdr_buffer.into_iter().flat_map(|v| v.to_array()).collect();
    let flat_albedo = albedo_buffer
        .into_iter()
        .flat_map(|v| v.to_array())
        .collect();
    let flat_normal = normal_buffer
        .into_iter()
        .flat_map(|v| v.to_array())
        .collect();
    (flat_hdr, flat_albedo, flat_normal)
}

pub fn save_hdr_image(
    hdr_data: &[f32],
    width: u32,
    height: u32,
    scene_path: &Path,
    suffix: &str,
    is_aov: bool,
    is_normal: bool,
    frame_number: usize
) -> ImageResult<()> {
    let path = Path::new(scene_path);
    let filename = path.file_stem().unwrap().to_str().unwrap();
    let output_dir = format!("output/{}", filename);
    let dir = Path::new(&output_dir);
    if !dir.exists() || !dir.is_dir() {
        create_dir(dir)
            .unwrap_or_else(|_| panic!("Failed to create output directory {}", dir.display()))
    }

    let mut img_buffer: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let i = ((y * width + x) * 3) as usize;
            let r_hdr = hdr_data[i];
            let g_hdr = hdr_data[i + 1];
            let b_hdr = hdr_data[i + 2];

            let mut color = Vec3A::new(r_hdr, g_hdr, b_hdr);

            if is_normal {
                // Remap normals from [-1, 1] to [0, 1] for saving
                color = color * 0.5 + 0.5;
            }

            if !is_aov {
                // Basic tone mapping
                color = color / (color + Vec3A::splat(1.0));
            }

            // Gamma correction
            color = color.powf(1.0 / 2.2);

            img_buffer.put_pixel(
                x,
                y,
                Rgb([
                    (color.x.clamp(0.0, 0.999) * 256.0) as u8,
                    (color.y.clamp(0.0, 0.999) * 256.0) as u8,
                    (color.z.clamp(0.0, 0.999) * 256.0) as u8,
                ]),
            );
        }
    }
    img_buffer.save(format!("output/{}/{}{}_{}.png", filename, filename, suffix, frame_number))
}

pub fn load_scene(
    path: &Path,
    aspect_ratio: f32,
) -> Result<Peekable<SceneIterator>, Box<dyn Error>> {
    info!("Loading scene from {:?}", path);

    // Scene
    let mut scene = loader::load_scene(path, aspect_ratio)?;
    if let Some(frame) = scene.peek() {
        info!("Loaded scene with {} objects", frame.objects.len());
    }
    Ok(scene)
}

pub fn load_and_save_scene(path: &Path, params: RenderParameters) -> Result<(), Box<dyn Error>> {
    let frames = load_scene(path, params.aspect_ratio)?;

    let mut last_save_task = None;
    // Create a channel to send scenes from the loading thread to the rendering thread.
    // The channel is bounded to 1 to avoid loading too many scenes into memory.
    let (scene_sender, scene_receiver) = std::sync::mpsc::sync_channel(1);

    // Spawn a thread to load scenes and send them to the rendering thread.
    let loading_thread = std::thread::spawn(move || {
        for scene in frames {
            if scene_sender.send(scene).is_err() {
                // The receiver has been dropped, so we can stop loading scenes.
                break;
            }
        }
    });

    for (i, scene) in scene_receiver.into_iter().enumerate() {
        let (rendered_hdr_data, albedo_data, normal_data) = render_hdr(params, scene);

        let path_clone = path.to_owned();
        last_save_task = Some(thread::spawn(move || {
            // Save albedo AOV for debugging
            save_hdr_image(
                &albedo_data,
                params.image_width,
                params.image_height,
                &path_clone,
                "_albedo",
                true,
                false,
                i,
            )
            .unwrap();

            // Save normal AOV for debugging
            save_hdr_image(
                &normal_data,
                params.image_width,
                params.image_height,
                &path_clone,
                "_normal",
                true,
                true,
                i,
            )
            .unwrap();

            // Save noisy HDR image (after sRGB conversion)
            save_hdr_image(
                &rendered_hdr_data,
                params.image_width,
                params.image_height,
                &path_clone,
                "_noisy",
                false,
                false,
                i,
            )
            .unwrap();

            denoise(
                params,
                &path_clone,
                rendered_hdr_data,
                albedo_data,
                normal_data,
                i,
            )
            .unwrap();
        }));
    }

    // Wait for the loading thread to finish.
    loading_thread.join().unwrap();
    last_save_task.unwrap().join().unwrap();

    Ok(())
}
#[cfg(feature = "denoise")]
fn denoise(
    params: RenderParameters,
    path: &Path,
    rendered_hdr_data: Vec<f32>,
    albedo_data: Vec<f32>,
    normal_data: Vec<f32>,
    frame_number: usize,
) -> Result<(), Box<dyn Error>> {
    // Denoise with OIDN
    info!("Denoising image with OIDN...");
    let mut denoised_hdr_data = vec![0.0; rendered_hdr_data.len()];
    let mut device = oidn::Device::new();
    oidn::RayTracing::new(&mut device)
        .image_dimensions(params.image_width as usize, params.image_height as usize)
        .hdr(true)
        .albedo_normal(&albedo_data, &normal_data)
        .filter(&rendered_hdr_data, &mut denoised_hdr_data)
        .map_err(|e| format!("{:?}", e))?;

    // Save denoised HDR image (after sRGB conversion)
    save_hdr_image(
        &denoised_hdr_data,
        params.image_width,
        params.image_height,
        path,
        "", // No suffix for the final denoised image
        false,
        false,
        frame_number
    )
    .map_err(Box::from)
}

#[cfg(not(feature = "denoise"))]
fn denoise(
    _params: RenderParameters,
    _path: &Path,
    _rendered_hdr_data: Vec<f32>,
    _albedo_data: Vec<f32>,
    _normal_data: Vec<f32>,
    _frame_number: usize
) -> Result<(), Box<dyn Error>> {
    warn!("Skipping final image generation: Denoising is disabled");
    Ok(())
}
