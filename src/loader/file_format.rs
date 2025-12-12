use glam::{Vec2, Vec3A};
use serde::{Deserialize, Serialize};
use splines::Interpolate;
use std::time::Duration;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Keyframe<T> {
    pub value: T,
    pub time: f32,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Default)]
pub struct AnimationSettings {
    pub duration: f32,
    pub frame_rate: u32,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Animation<T> {
    pub interpolation: splines::Interpolation<f32, f32>,
    pub keyframes: Vec<Keyframe<T>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum Animatable<T> {
    Static(T),
    Animated(Animation<T>),
}

impl<T: Default> Default for Animatable<T> {
    fn default() -> Self {
        Animatable::Static(T::default())
    }
}

#[derive(Debug, Default)]
pub struct Background {
    pub color: Vec3A,
    pub ambient_light: Vec3A,
}

#[derive(Debug, Default)]
pub struct Camera {
    pub eye: Animatable<Vec3A>,
    pub look_at: Animatable<Vec3A>,
    pub up: Animatable<Vec3A>,
    pub fovy: Animatable<f32>,
}

#[derive(Debug, Default)]
pub struct Light {
    pub position: Animatable<Vec3A>,
    pub color: Animatable<Vec3A>,
}

#[derive(Debug, Clone, Default)]
pub struct Material {
    pub texture_filename: Option<String>,
    pub normal_map_filename: Option<String>,
    pub displacement_map_filename: Option<String>,
    pub diffuse_color: Animatable<Vec3A>,
    pub specular_color: Animatable<Vec3A>,
    pub reflective_color: Animatable<Vec3A>,
    pub shininess: Animatable<f32>,
    pub transparent_color: Animatable<Vec3A>,
    pub index_of_refraction: Animatable<f32>,
    pub displacement_strength: Animatable<f32>,
    pub subdivision_level: Option<u32>,
    pub max_edge_length: Option<f32>,
    pub emissive_color: Option<Animatable<Vec3A>>,
}

#[derive(Debug)]
pub enum Object {
    Sphere {
        material_index: usize,
        center: Animatable<Vec3A>,
        radius: Animatable<f32>,
    },
    Triangle {
        vertex0: Animatable<Vec3A>,
        vertex1: Animatable<Vec3A>,
        vertex2: Animatable<Vec3A>,
        tex_xy_0: Animatable<Vec2>,
        tex_xy_1: Animatable<Vec2>,
        tex_xy_2: Animatable<Vec2>,
        normal0: Animatable<Vec3A>,
        normal1: Animatable<Vec3A>,
        normal2: Animatable<Vec3A>,
        material_index: usize,
    },
    Mesh {
        filename: String,
        material_index: usize,
    },
    Cylinder {
        center: Animatable<Vec3A>,
        radius: Animatable<f32>,
        height: Animatable<f32>,
        material_index: usize,
    },
}

#[derive(Debug, Default)]
pub struct Scene {
    pub background: Background,
    pub camera: Camera,
    pub lights: Vec<Light>,
    pub materials: Vec<Material>,
    pub objects: Vec<Object>,
    pub animation_settings: AnimationSettings,
}
