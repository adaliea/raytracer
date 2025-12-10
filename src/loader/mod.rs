mod file_format;
mod parser;

use crate::camera::Camera;
use crate::hittable::HittableObject;
use crate::hittable::sphere::Sphere;
use crate::hittable::triangle::Triangle;
use crate::hittable::tessellator;
use crate::material::{Material, Texture};
use crate::scene::Scene;
use bvh::bounding_hierarchy::BoundingHierarchy;
use bvh::bvh::Bvh;
use glam::Vec3A;
use image::{ImageError, Rgb32FImage};
use log::{error, info, warn};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::Arc;

fn warn_out_of_range(range: (f32, f32), value: f32, name: &str) -> f32 {
    if value > range.1 {
        warn!(
            "{} is out of range; expected <= {}; got {}",
            name, range.1, value
        );
        range.1
    } else if value < range.0 {
        warn!(
            "{} is out of range; expected >= {}; got {}",
            name, range.0, value
        );
        range.0
    } else {
        value
    }
}

fn load_texture(texture_path: &Path, scene_path: &Path) -> Result<Rgb32FImage, ImageError> {
    let texture_path = scene_path.parent().unwrap().join(texture_path);
    info!("Attempting to load texture at: {:?}", &texture_path);
    let img = image::open(&texture_path);
    img.map(|i| i.to_rgb32f())
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

    let mut materials: Vec<Material> = Vec::new();
    let mut objects: Vec<HittableObject> = Vec::new();

    for mat in file_scene.materials {
        let fuzz = 1.0 - (warn_out_of_range((0.0, 100.0), mat.shininess, "shininess") / 100.0);

        // Load albedo texture if it exists
        let albedo = if let Some(filename) = mat.texture_filename.filter(|f| f != "NULL") {
            load_texture(Path::new(&filename), path).map_or_else(
                |error| {
                    error!(
                        "Failed to load texture {} in {}; reason: {}",
                        filename,
                        path.display(),
                        error
                    );
                    // Use diffuse color as fallback
                    Texture::SolidColor(if mat.reflective_color.length() > 0.0 {
                        mat.reflective_color
                    } else {
                        mat.diffuse_color
                    })
                },
                |i| Texture::Image(Arc::new(i)),
            )
        } else if mat.reflective_color.length() > 0.0 {
            Texture::SolidColor(mat.reflective_color)
        } else {
            Texture::SolidColor(mat.diffuse_color)
        };

        // Load normal map if it exists
        let normal_map = if let Some(filename) = mat.normal_map_filename.filter(|f| f != "NULL") {
            load_texture(Path::new(&filename), path).map_or_else(
                |error| {
                    error!(
                        "Failed to load normal map {} in {}; reason: {}",
                        filename,
                        path.display(),
                        error
                    );
                    None
                },
                |i| Some(Texture::Image(Arc::new(i))),
            )
        } else {
            None
        };

        // Load displacement map if it exists
        let displacement_map =
            if let Some(filename) = mat.displacement_map_filename.filter(|f| f != "NULL") {
                load_texture(Path::new(&filename), path).map_or_else(
                    |error| {
                        error!(
                            "Failed to load displacement map {} in {}; reason: {}",
                            filename,
                            path.display(),
                            error
                        );
                        None
                    },
                    |i| Some(Texture::Image(Arc::new(i))),
                )
            } else {
                None
            };


        let material = if mat.transparent_color.length() > 0.0 {
            Material::Dielectric {
                index_of_refraction: mat.index_of_refraction,
                fuzz,
                displacement_map,
                displacement_strength: mat.displacement_strength,
                subdivision_level: mat.subdivision_level,
                max_edge_length: mat.max_edge_length,
            }
        } else {
            if mat.reflective_color.length() > 0.0 {
                Material::Metallic {
                    albedo,
                    fuzz,
                    normal_map,
                    displacement_map,
                    displacement_strength: mat.displacement_strength,
                    subdivision_level: mat.subdivision_level,
                    max_edge_length: mat.max_edge_length,
                }
            } else {
                Material::Lambertian {
                    albedo,
                    normal_map,
                    displacement_map,
                    displacement_strength: mat.displacement_strength,
                    subdivision_level: mat.subdivision_level,
                    max_edge_length: mat.max_edge_length,
                }
            }
        };

        materials.push(material);
    }

    if materials.is_empty() {
        materials.push(Material::Lambertian {
            albedo: Texture::SolidColor(Vec3A::splat(0.5)),
            normal_map: None,
            displacement_map: None,
            displacement_strength: 0.0,
            subdivision_level: None,
            max_edge_length: None,
        });
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
                normal0,
                normal1,
                normal2,
                material_index,
            } => {
                let material = if material_index < materials.len() {
                    materials[material_index].clone()
                } else {
                    materials[0].clone()
                };

                let triangle = Triangle::new(
                    vertex0, vertex1, vertex2, tex_xy_0, tex_xy_1, tex_xy_2, normal0, normal1,
                    normal2, material.clone(),
                );

                if material.has_displacement_map() {
                    let (subdivision_level, max_edge_length, displacement_strength) = match &material {
                        Material::Lambertian { subdivision_level, max_edge_length, displacement_strength, .. } => (*subdivision_level, *max_edge_length, *displacement_strength),
                        Material::Metallic { subdivision_level, max_edge_length, displacement_strength, .. } => (*subdivision_level, *max_edge_length, *displacement_strength),
                        Material::Dielectric { subdivision_level, max_edge_length, displacement_strength, .. } => (*subdivision_level, *max_edge_length, *displacement_strength),
                        Material::Emissive { .. } => (None, None, 0.0),
                    };

                    let tessellated_triangles = tessellator::tessellate_triangle(
                        &triangle,
                        subdivision_level,
                        max_edge_length,
                        &material,
                        displacement_strength,
                    );
                    HittableObject::Mesh(crate::hittable::mesh::Mesh::new(tessellated_triangles))
                } else {
                    HittableObject::Triangle(triangle)
                }
            }
            file_format::Object::Mesh {
                filename,
                material_index,
            } => {
                let _material = if material_index < materials.len() {
                    materials[material_index].clone()
                } else {
                    materials[0].clone()
                };

                // Placeholder for OBJ loading. This will be implemented later.
                // For now, return an empty mesh or handle as an error.
                warn!("OBJ loading not yet implemented. Returning an empty mesh for {}", filename);
                HittableObject::Mesh(crate::hittable::mesh::Mesh::new(vec![]))
            }
            file_format::Object::Cylinder {
                center,
                radius,
                height,
                material_index,
            } => {
                let material = if material_index < materials.len() {
                    materials[material_index].clone()
                } else {
                    materials[0].clone()
                };

                // Define a default number of segments for cylinder tessellation
                let segments = 32;

                let initial_triangles =
                    crate::hittable::mesh::Mesh::generate_cylinder_triangles(
                        center, radius, height, segments, material.clone(),
                    );

                let mut final_triangles = Vec::new();
                
                let (subdivision_level, max_edge_length, displacement_strength) = match &material {
                    Material::Lambertian { subdivision_level, max_edge_length, displacement_strength, .. } => (*subdivision_level, *max_edge_length, *displacement_strength),
                    Material::Metallic { subdivision_level, max_edge_length, displacement_strength, .. } => (*subdivision_level, *max_edge_length, *displacement_strength),
                    Material::Dielectric { subdivision_level, max_edge_length, displacement_strength, .. } => (*subdivision_level, *max_edge_length, *displacement_strength),
                    Material::Emissive { .. } => (None, None, 0.0),
                };

                for tri in initial_triangles {
                    let material_from_tri = &tri.material;

                    if material_from_tri.has_displacement_map() {
                        let tessellated = tessellator::tessellate_triangle(
                            &tri,
                            subdivision_level,
                            max_edge_length,
                            material_from_tri,
                            displacement_strength,
                        );
                        final_triangles.extend(tessellated);
                    } else {
                        final_triangles.push(tri);
                    }
                }

                HittableObject::Mesh(crate::hittable::mesh::Mesh::new(final_triangles))
            }
        };

        objects.push(obj);
    }

    let default_light_strength = 50.0;
    let default_light_radius = 1.0;
    let mut lights: Vec<usize> = Vec::new();

    for light in file_scene.lights {
        let emissive_material = Material::Emissive {
            emit_color: Texture::SolidColor(light.color),
            strength: default_light_strength,
        };

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