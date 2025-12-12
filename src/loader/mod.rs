use crate::camera::Camera;
use crate::hittable::{Hittable, HittableObject};
use crate::loader::file_format::{Animatable, Animation};
use crate::loader::shapes::generate_cylinder_triangles;
use crate::material::{Material, Texture};
use crate::scene::Scene;
use bvh::bvh::Bvh;
use glam::{FloatExt, Vec2, Vec3A};
use image::{DynamicImage, ImageError, Rgb32FImage};
use log::{error, info, log, warn};
use splines::{Interpolate, Spline};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::iter::Peekable;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};

mod file_format;
mod parser;
mod shapes;

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

static TEXTURE_CACHE: Mutex<LazyLock<HashMap<PathBuf, Arc<Rgb32FImage>>>> =
    Mutex::new(LazyLock::new(|| HashMap::new()));

fn load_texture(texture_path: &Path, scene_path: &Path) -> Result<Arc<Rgb32FImage>, ImageError> {
    let texture_path = scene_path.parent().unwrap().join(texture_path);
    let mut cache = TEXTURE_CACHE.lock().unwrap();
    if let Some(image) = cache.get(&texture_path) {
        Ok(image.clone())
    } else {
        info!("Attempting to load texture at: {:?}", &texture_path);
        let img = image::open(&texture_path).map(|i| i.to_rgb32f())?;
        let arc = Arc::new(img);
        cache.insert(texture_path.to_owned(), arc.clone());
        Ok(arc)
    }
}

fn resolve_animatable_vec_3(animatable: &Animatable<Vec3A>, time: f32) -> Vec3A {
    resolve_animatable(
        animatable,
        time,
        |vec: Vec3A, index| vec[index],
        |val, index, vec| vec[index] = val,
    )
}

fn resolve_animatable_vec_2(animatable: &Animatable<Vec2>, time: f32) -> Vec2 {
    resolve_animatable(
        animatable,
        time,
        |vec: Vec2, index| vec[index],
        |val, index, vec| vec[index] = val,
    )
}

fn resolve_animatable_f32(animatable: &Animatable<f32>, time: f32) -> f32 {
    resolve_animatable(animatable, time, |f, _index| f, |val, _index, f| *f = val)
}
fn resolve_animatable<T: Copy + Default, A: Fn(T, usize) -> f32, B: Fn(f32, usize, &mut T)>(
    animatable: &Animatable<T>,
    time: f32,
    into_f32: A,
    from_f32: B,
) -> T {
    match animatable {
        Animatable::Static(value) => *value,
        Animatable::Animated(animation) => {
            if animation.keyframes.is_empty() {
                return T::default();
            }
            if time <= animation.keyframes.first().unwrap().time {
                return animation.keyframes.first().unwrap().value;
            }
            if time >= animation.keyframes.last().unwrap().time {
                return animation.keyframes.last().unwrap().value;
            }

            let mut output = T::default();
            // Interpolate each dimension independently
            for i in 0..2 {
                let keys = animation
                    .keyframes
                    .iter()
                    .map(|k| (k.time, into_f32(k.value, i)))
                    .map(|kf| splines::Key::new(kf.0, kf.1, splines::Interpolation::Linear))
                    .collect();

                let spline = Spline::from_vec(keys);
                let sample = spline.sample(time).unwrap();
                from_f32(sample, i, &mut output);
            }
            output
        }
    }
}

pub fn load_scene_at_time(
    path: &Path,
    file_scene: &file_format::Scene,
    aspect_ratio: f32,
    time: f32,
) -> Result<Scene, Box<dyn Error>> {
    let camera = Camera::new(
        resolve_animatable_vec_3(&file_scene.camera.eye, time),
        resolve_animatable_vec_3(&file_scene.camera.look_at, time),
        resolve_animatable_vec_3(&file_scene.camera.up, time),
        resolve_animatable_f32(&file_scene.camera.fovy, time),
        aspect_ratio,
    );

    let mut materials: Vec<Material> = Vec::new();
    let mut objects: Vec<HittableObject> = Vec::new();

    for mat in &file_scene.materials {
        let shininess = resolve_animatable_f32(&mat.shininess, time);
        let fuzz = 1.0 - (warn_out_of_range((0.0, 100.0), shininess, "shininess") / 100.0);

        let material = if let Some(emit_color) = &mat.emissive_color {
            Material::Emissive {
                emit_color: Texture::SolidColor(resolve_animatable_vec_3(&emit_color, time)),
                strength: 1.0, // Default strength
            }
        } else {
            let displacement_map = mat.displacement_map_filename.as_ref().and_then(|filename| {
                if filename != "NULL" {
                    load_texture(Path::new(filename), path)
                        .map(|i| Texture::Image(i))
                        .inspect_err(|e| {
                            warn!(
                                "Failed to load displacement map: {}, error: {:?}",
                                filename, e
                            )
                        })
                        .ok()
                } else {
                    None
                }
            });

            let transparent_color = resolve_animatable_vec_3(&mat.transparent_color, time);
            if transparent_color.length() > 0.0 {
                Material::Dielectric {
                    index_of_refraction: resolve_animatable_f32(&mat.index_of_refraction, time),
                    fuzz,
                    displacement_map,
                    displacement_strength: resolve_animatable_f32(&mat.displacement_strength, time),
                    subdivision_level: mat.subdivision_level,
                    max_edge_length: mat.max_edge_length,
                }
            } else {
                let reflective_color = resolve_animatable_vec_3(&mat.reflective_color, time);
                let albedo = mat
                    .texture_filename
                    .as_ref()
                    .and_then(|filename| {
                        if filename != "NULL" {
                            load_texture(Path::new(filename), path)
                                .map(|i| Texture::Image(i))
                                .inspect_err(|e| {
                                    warn!("Failed to load texture: {}, error: {:?}", filename, e)
                                })
                                .ok()
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| {
                        if reflective_color.length() > 0.0 {
                            Texture::SolidColor(reflective_color)
                        } else {
                            Texture::SolidColor(resolve_animatable_vec_3(&mat.diffuse_color, time))
                        }
                    });

                let normal_map = mat.normal_map_filename.as_ref().and_then(|filename| {
                    if filename != "NULL" {
                        load_texture(Path::new(filename), path)
                            .map(|i| Texture::Image(i))
                            .inspect_err(|e| {
                                warn!("Failed to load normal map: {}, error: {:?}", filename, e)
                            })
                            .ok()
                    } else {
                        None
                    }
                });

                if reflective_color.length() > 0.0 {
                    Material::Metallic {
                        albedo,
                        fuzz,
                        normal_map,
                        displacement_map,
                        displacement_strength: resolve_animatable_f32(
                            &mat.displacement_strength,
                            time,
                        ),
                        subdivision_level: mat.subdivision_level,
                        max_edge_length: mat.max_edge_length,
                    }
                } else {
                    Material::Lambertian {
                        albedo,
                        normal_map,
                        displacement_map,
                        displacement_strength: resolve_animatable_f32(
                            &mat.displacement_strength,
                            time,
                        ),
                        subdivision_level: mat.subdivision_level,
                        max_edge_length: mat.max_edge_length,
                    }
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

    for obj in &file_scene.objects {
        match obj {
            file_format::Object::Sphere {
                material_index,
                center,
                radius,
            } => {
                let material = if *material_index < materials.len() {
                    materials[*material_index].clone()
                } else {
                    materials[0].clone()
                };

                objects.push(HittableObject::Sphere(
                    crate::hittable::sphere::Sphere::new(
                        resolve_animatable_vec_3(&center, time),
                        resolve_animatable_f32(&radius, time),
                        material,
                    ),
                ));
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
                let material = if *material_index < materials.len() {
                    materials[*material_index].clone()
                } else {
                    materials[0].clone()
                };

                let triangle = crate::hittable::triangle::Triangle::new(
                    resolve_animatable_vec_3(&vertex0, time),
                    resolve_animatable_vec_3(&vertex1, time),
                    resolve_animatable_vec_3(&vertex2, time),
                    resolve_animatable_vec_2(&tex_xy_0, time),
                    resolve_animatable_vec_2(&tex_xy_1, time),
                    resolve_animatable_vec_2(&tex_xy_2, time),
                    resolve_animatable_vec_3(&normal0, time),
                    resolve_animatable_vec_3(&normal1, time),
                    resolve_animatable_vec_3(&normal2, time),
                    material.clone(),
                );

                if material.has_displacement_map() {
                    let (subdivision_level, max_edge_length, displacement_strength) =
                        match &material {
                            Material::Lambertian {
                                subdivision_level,
                                max_edge_length,
                                displacement_strength,
                                ..
                            } => (*subdivision_level, *max_edge_length, *displacement_strength),
                            Material::Metallic {
                                subdivision_level,
                                max_edge_length,
                                displacement_strength,
                                ..
                            } => (*subdivision_level, *max_edge_length, *displacement_strength),
                            Material::Dielectric {
                                subdivision_level,
                                max_edge_length,
                                displacement_strength,
                                ..
                            } => (*subdivision_level, *max_edge_length, *displacement_strength),
                            Material::Emissive { .. } => (None, None, 0.0),
                        };

                    let tessellated_triangles = crate::hittable::tessellator::tessellate_triangle(
                        &triangle,
                        subdivision_level,
                        max_edge_length,
                        &material,
                        displacement_strength,
                    );
                    tessellated_triangles.into_iter().for_each(|t| {
                        objects.push(HittableObject::Triangle(t));
                    });
                } else {
                    objects.push(HittableObject::Triangle(triangle));
                }
            }
            file_format::Object::Mesh {
                filename,
                material_index,
            } => {
                let _material = if *material_index < materials.len() {
                    materials[*material_index].clone()
                } else {
                    materials[0].clone()
                };

                // Placeholder for OBJ loading. This will be implemented later.
                warn!(
                    "OBJ loading not yet implemented. Returning an empty mesh for {}",
                    filename
                );
            }
            file_format::Object::Cylinder {
                center,
                radius,
                height,
                material_index,
            } => {
                let material = if *material_index < materials.len() {
                    materials[*material_index].clone()
                } else {
                    materials[0].clone()
                };

                // Define a default number of segments for cylinder tessellation
                let segments = 32;

                let initial_triangles = generate_cylinder_triangles(
                    resolve_animatable_vec_3(&center, time),
                    resolve_animatable_f32(&radius, time),
                    resolve_animatable_f32(&height, time),
                    segments,
                    material.clone(),
                );

                let mut final_triangles = Vec::new();

                let (subdivision_level, max_edge_length, displacement_strength) = match &material {
                    Material::Lambertian {
                        subdivision_level,
                        max_edge_length,
                        displacement_strength,
                        ..
                    } => (*subdivision_level, *max_edge_length, *displacement_strength),
                    Material::Metallic {
                        subdivision_level,
                        max_edge_length,
                        displacement_strength,
                        ..
                    } => (*subdivision_level, *max_edge_length, *displacement_strength),
                    Material::Dielectric {
                        subdivision_level,
                        max_edge_length,
                        displacement_strength,
                        ..
                    } => (*subdivision_level, *max_edge_length, *displacement_strength),
                    Material::Emissive { .. } => (None, None, 0.0),
                };

                for tri in initial_triangles {
                    let material_from_tri = &tri.material;

                    if material_from_tri.has_displacement_map() {
                        let tessellated = crate::hittable::tessellator::tessellate_triangle(
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

                final_triangles.into_iter().for_each(|t| {
                    objects.push(HittableObject::Triangle(t));
                })
            }
        };
    }

    let default_light_strength = 50.0;
    let default_light_radius = 1.0;
    let mut lights: Vec<usize> = Vec::new();

    for light in &file_scene.lights {
        let emissive_material = Material::Emissive {
            emit_color: Texture::SolidColor(resolve_animatable_vec_3(&light.color, time)),
            strength: default_light_strength,
        };

        materials.push(emissive_material.clone());

        let light_sphere = HittableObject::Sphere(crate::hittable::sphere::Sphere::new(
            resolve_animatable_vec_3(&light.position, time),
            default_light_radius,
            emissive_material,
        ));

        objects.push(light_sphere);
        lights.push(objects.len() - 1);
    }

    let bvh = Bvh::build(&mut objects);

    let scene = Scene {
        camera,
        objects,
        lights,
        bvh,
        background_color: file_scene.background.color,
    };

    Ok(scene)
}

pub struct SceneIterator {
    file_scene: file_format::Scene,
    aspect_ratio: f32,
    path: PathBuf,
    frame_number: usize,
}

impl Iterator for SceneIterator {
    type Item = Scene;
    fn next(&mut self) -> Option<Self::Item> {
        let time = if self.file_scene.animation_settings.frame_rate > 0 {
            self.frame_number as f32 / self.file_scene.animation_settings.frame_rate as f32
        } else {
            0.0
        };
        if self.frame_number > 0 && time == 0.0 {
            return None; // No animations settings & we already rendered a frame
        }
        self.frame_number += 1;
        if time > self.file_scene.animation_settings.duration {
            return None;
        }
        load_scene_at_time(&self.path, &self.file_scene, self.aspect_ratio, time)
            .inspect_err(|err| error!("Error loading scene {:?}", err))
            .ok()
    }
}

impl ExactSizeIterator for SceneIterator {}
pub(crate) fn load_scene(
    path: &Path,
    aspect_ratio: f32,
) -> Result<Peekable<SceneIterator>, Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;
    let file_scene = parser::parse_ray_file(&contents);
    info!("{:?}, {:?}", &contents, &file_scene);
    Ok(SceneIterator {
        file_scene,
        aspect_ratio,
        path: Default::default(),
        frame_number: 0,
    }
    .peekable())
}
