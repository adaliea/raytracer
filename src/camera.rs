use crate::ray::Ray;
use glam::Vec3A;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub origin: Vec3A,
    pub lower_left_corner: Vec3A,
    pub horizontal: Vec3A,
    pub vertical: Vec3A,
}

impl Camera {
    pub fn new(look_from: Vec3A, look_at: Vec3A, vup: Vec3A, vfov: f32, aspect_ratio: f32) -> Self {
        let theta = vfov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;

        let w = (look_from - look_at).normalize();
        let u = vup.cross(w).normalize();
        let v = w.cross(u);

        let origin = look_from;
        let horizontal = viewport_width * u;
        let vertical = viewport_height * v;
        let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - w;

        Self {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
        }
    }

    pub fn get_ray(&self, u: f32, v: f32) -> Ray {
        Ray {
            origin: self.origin,
            direction: self.lower_left_corner + u * self.horizontal + v * self.vertical
                - self.origin,
        }
    }
}
