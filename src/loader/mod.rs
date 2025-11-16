mod file_format;
mod parser;

use crate::camera::Camera;
use crate::hittable::{HittableObject, Sphere};
use crate::material::Material;
use crate::scene::Scene;
use image::RgbImage;
use log::{debug, error};
use std::fs;

fn load_texture(texture_path: &str, scene_path: &str) -> RgbImage {
    let texture_path = std::path::Path::new(scene_path)
        .parent()
        .unwrap()
        .join(texture_path);
    let img = image::open(&texture_path);
    if !img.is_ok() {
        error!("Texture not found: {}", &texture_path.display());
    }
    img.map(|i| i.to_rgb8())
        .unwrap_or_else(|_| RgbImage::from_pixel(1, 1, image::Rgb([255, 255, 255])))
}

pub fn load_scene(path: &str, aspect_ratio: f32) -> Result<Scene, std::io::Error> {
    let contents = fs::read_to_string(path)?;
    let file_scene = parser::parse_ray_file(&contents);
    debug!("{:#?}", file_scene);

    let camera = Camera::new(
        file_scene.camera.eye,
        file_scene.camera.look_at,
        file_scene.camera.up,
        file_scene.camera.fovy,
        aspect_ratio,
    );

    let materials: Vec<Material> = file_scene
        .materials
        .into_iter()
        .enumerate()
        .map(|(_i, mat)| {
            let texture = if mat.texture_filename.is_some() {
                load_texture(mat.texture_filename.as_ref().unwrap(), path)
            } else {
                RgbImage::from_pixel(1, 1, image::Rgb([255, 255, 255]))
            };
            debug!("Loading mat: {:#?}", &mat);
            Material {
                texture,
                diffuse_color: mat.diffuse_color,
                specular_color: mat.specular_color,
                reflective_color: mat.reflective_color,
                metallicity: 0.0,
                shininess: mat.shininess,
                transparent_color: mat.transparent_color,
                refractive_index: mat.index_of_refraction,
            }
        })
        .collect();

    let objects: Vec<HittableObject> = file_scene
        .objects
        .into_iter()
        .map(|obj| match obj {
            file_format::Object::Sphere {
                material_index,
                center,
                radius,
            } => {
                let valid_index = if (material_index + 1) < materials.len() {
                    material_index + 1
                } else {
                    0
                };

                HittableObject::Sphere(Sphere {
                    center,
                    radius,
                    material_index: valid_index,
                })
            }
        })
        .collect();

    let scene = Scene {
        camera,
        objects,
        materials,
    };
    debug!("{:#?}", scene);

    Ok(scene)
}
