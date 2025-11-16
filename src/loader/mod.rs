mod file_format;
mod parser;

use crate::camera::Camera;
use crate::hittable::{HittableObject, Sphere, Triangle};
use crate::material::{Material, Texture};
use crate::scene::Scene;
use glam::Vec3A;
use image::{ImageError, RgbImage};
use log::{debug, warn};
use std::fs;
use std::sync::Arc;

fn load_texture(texture_path: &str, scene_path: &str) -> Result<RgbImage, ImageError> {
    let texture_path = std::path::Path::new(scene_path)
        .parent()
        .unwrap()
        .join(texture_path);
    let img = image::open(&texture_path);
    img.map(|i| i.to_rgb8())
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

    let mut materials: Vec<Arc<Material>> = Vec::new();
    let mut objects: Vec<HittableObject> = Vec::new();

    for mat in file_scene.materials {
        let fuzz = (1.0 - (mat.shininess / 100.0)).max(0.0).min(1.0);

        let material = if mat.transparent_color.length() > 0.0 {
            Material::Dielectric {
                index_of_refraction: mat.index_of_refraction,
                fuzz,
            }
        } else if mat.reflective_color.length() > 0.0 {
            Material::Metallic {
                albedo: Texture::SolidColor(mat.reflective_color),
                fuzz,
            }
        } else {
            // Check for texture
            let albedo = if let Some(filename) = mat.texture_filename.filter(|f| f != "NULL") {
                Texture::Image(
                    load_texture(&filename, path)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
                )
            } else {
                Texture::SolidColor(mat.diffuse_color)
            };
            Material::Lambertian { albedo }
        };

        materials.push(Arc::new(material));
    }

    if materials.is_empty() {
        materials.push(Arc::new(Material::Lambertian {
            albedo: Texture::SolidColor(Vec3A::splat(0.5)),
        }));
        warn!("No materials found, using default");
    }

    for obj in file_scene.objects {
        let obj = match obj {
            file_format::Object::Sphere {
                material_index,
                center,
                radius,
            } => {
                let material = if material_index < materials.len() {
                    materials[material_index].clone()
                } else {
                    materials[0].clone()
                };

                HittableObject::Sphere(Sphere {
                    center,
                    radius,
                    material,
                })
            }
            file_format::Object::Triangle {
                vertex0,
                vertex1,
                vertex2,
                tex_xy_0,
                tex_xy_1,
                tex_xy_2,
                material_index,
            } => {
                let material = if material_index < materials.len() {
                    materials[material_index].clone()
                } else {
                    materials[0].clone()
                };

                HittableObject::Triangle(Triangle {
                    v0: vertex0,
                    v1: vertex1,
                    v2: vertex2,
                    uv0: tex_xy_0,
                    uv1: tex_xy_1,
                    uv2: tex_xy_2,
                    material,
                })
            }
        };

        objects.push(obj);
    }

    let default_light_strength = 10.0;
    let default_light_radius = 0.25;
    let mut lights: Vec<usize> = Vec::new();

    for light in file_scene.lights {
        let emissive_material = Arc::new(Material::Emissive {
            emit_color: Texture::SolidColor(light.color),
            strength: default_light_strength,
        });

        materials.push(emissive_material.clone());

        let light_sphere = HittableObject::Sphere(Sphere {
            center: light.position,
            radius: default_light_radius,
            material: emissive_material,
        });

        objects.push(light_sphere);
        lights.push(objects.len() - 1);
    }

    let scene = Scene {
        camera,
        objects,
        lights,
        background_color: file_scene.background.color,
    };
    debug!("{:#?}", scene);

    Ok(scene)
}
