mod parser;
mod file_format;

use std::fs;
use crate::camera::Camera;
use crate::hittable::{HittableObject, Sphere};
use crate::material::Material;
use crate::scene::Scene;

pub fn load_scene(path: &str, aspect_ratio: f32) -> Result<Scene, std::io::Error> {
    let contents = fs::read_to_string(path)?;
    let file_scene = parser::parse_ray_file(&contents);

    let camera = Camera::new(
        file_scene.camera.eye,
        file_scene.camera.look_at,
        file_scene.camera.up,
        file_scene.camera.fovy,
        aspect_ratio,
    );

    let objects = file_scene.objects.into_iter().map(|obj| {
        match obj {
            file_format::Object::Sphere { material_index, center, radius } => {
                let material = file_scene.materials.get(material_index).cloned().map_or(
                    Material::Lambertian { albedo: glam::Vec3A::new(0.5, 0.5, 0.5) }, // Default material
                    |m| Material::Lambertian { albedo: m.diffuse_color }
                );
                HittableObject::Sphere(Sphere {
                    center,
                    radius,
                    material,
                })
            }
        }
    }).collect();

    Ok(Scene {
        camera,
        objects,
    })
}