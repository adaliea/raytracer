use glam::{Vec2, Vec3A};

#[derive(Debug, Default)]
pub struct Background {
    pub color: Vec3A,
    pub ambient_light: Vec3A,
}

#[derive(Debug, Default)]
pub struct Camera {
    pub eye: Vec3A,
    pub look_at: Vec3A,
    pub up: Vec3A,
    pub fovy: f32,
}

#[derive(Debug, Default)]
pub struct Light {
    pub position: Vec3A,
    pub color: Vec3A,
}

#[derive(Debug, Default, Clone)]
pub struct Material {
    pub texture_filename: Option<String>,
    pub normal_map_filename: Option<String>,
    pub displacement_map_filename: Option<String>,
    pub diffuse_color: Vec3A,
    pub specular_color: Vec3A,
    pub reflective_color: Vec3A,
    pub shininess: f32,
    pub transparent_color: Vec3A,
    pub index_of_refraction: f32,
    pub displacement_strength: f32,
}

#[derive(Debug)]
pub enum Object {
    Sphere {
        material_index: usize,
        center: Vec3A,
        radius: f32,
    },
    Triangle {
        vertex0: Vec3A,
        vertex1: Vec3A,
        vertex2: Vec3A,
        tex_xy_0: Vec2,
        tex_xy_1: Vec2,
        tex_xy_2: Vec2,
        normal0: Vec3A,
        normal1: Vec3A,
        normal2: Vec3A,
        material_index: usize,
    },
    Mesh {
        filename: String,
        material_index: usize,
    },
    Cylinder {
        center: Vec3A,
        radius: f32,
        height: f32,
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
}
