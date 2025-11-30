mod file_format;
mod parser;

use crate::camera::Camera;
use crate::hittable::HittableObject;
use crate::hittable::sphere::Sphere;
use crate::hittable::triangle::Triangle;
use crate::material::{Material, Texture};
use crate::scene::Scene;
use bvh::bounding_hierarchy::BoundingHierarchy;
use bvh::bvh::Bvh;
use glam::Vec3A;
use image::{ImageError, RgbImage};
use log::{error, warn};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::Arc;

fn load_texture(texture_path: &Path, scene_path: &Path) -> Result<RgbImage, ImageError> {
    let texture_path = scene_path.parent().unwrap().join(texture_path);
    let img = image::open(&texture_path);
    img.map(|i| i.to_rgb8())
}

pub fn load_scene(path: &Path, aspect_ratio: f32) -> Result<Scene, Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;
    let file_scene = parser::parse_ray_file(&contents);

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
        let fuzz = (1.0 - (mat.shininess / 100.0)).clamp(0.0, 1.0);

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
                load_texture(&Path::new(&filename), path).map_or_else(
                    |error| {
                        error!(
                            "Failed to load texture {} in {}; reason: {}",
                            filename,
                            path.display(),
                            error
                        );
                        Texture::SolidColor(mat.diffuse_color)
                    },
                    Texture::Image,
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

                HittableObject::Sphere(Sphere::new(center, radius, material))
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

                HittableObject::Triangle(Triangle::new(
                    vertex0, vertex1, vertex2, tex_xy_0, tex_xy_1, tex_xy_2, material,
                ))
            }
        };

        objects.push(obj);
    }

    let default_light_strength = 50.0;
    let default_light_radius = 1.0;
    let mut lights: Vec<usize> = Vec::new();

    for light in file_scene.lights {
        let emissive_material = Arc::new(Material::Emissive {
            emit_color: Texture::SolidColor(light.color),
            strength: default_light_strength,
        });

        materials.push(emissive_material.clone());

        let light_sphere = HittableObject::Sphere(Sphere::new(
            light.position,
            default_light_radius,
            emissive_material,
        ));

        objects.push(light_sphere);
        lights.push(objects.len() - 1);
    }

    let bvh = Bvh::build_par(&mut objects);

    let scene = Scene {
        camera,
        objects,
        lights,
        bvh,
        background_color: file_scene.background.color,
    };

    //debug!("Loaded scene: {:?}", scene);

    Ok(scene)
}
