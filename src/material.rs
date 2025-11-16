use glam::Vec3A;

#[derive(Debug, Clone)]
pub enum Material {
    Lambertian { albedo: Vec3A },
    Metal { albedo: Vec3A, fuzz: f32 },
}
