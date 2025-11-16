use glam::Vec3A;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Material {
    Lambertian { albedo: Vec3A },
    Metal { albedo: Vec3A, fuzz: f32 },
}
