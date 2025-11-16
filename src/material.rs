use glam::Vec3A;
use image::RgbImage;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Material {
    pub texture: RgbImage,
    pub diffuse_color: Vec3A,
    pub specular_color: Vec3A,
    pub reflective_color: Vec3A,
    pub shininess: f32,
    pub metallicity: f32,
    pub refractive_index: f32,
    pub transparent_color: Vec3A,
}
