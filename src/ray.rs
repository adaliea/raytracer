use glam::Vec3A;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vec3A,
    pub direction: Vec3A,
}

impl Ray {
    pub fn at(&self, t: f32) -> Vec3A {
        self.origin + t * self.direction
    }
}
