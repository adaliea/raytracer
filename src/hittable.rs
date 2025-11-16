use glam::Vec3A;
use crate::ray::Ray;
use crate::material::Material;

#[derive(Debug, Clone)]
pub struct HitRecord {
    pub p: Vec3A,
    pub normal: Vec3A,
    pub t: f32,
    pub front_face: bool,
    pub material: Material,
}

impl HitRecord {
    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: Vec3A) {
        self.front_face = r.direction.dot(outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }
}

pub trait Hittable: Send + Sync {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}

#[derive(Debug, Clone)]
pub enum HittableObject {
    Sphere(Sphere),
}

impl Hittable for HittableObject {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        match self {
            HittableObject::Sphere(s) => s.hit(r, t_min, t_max),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Vec3A,
    pub radius: f32,
    pub material: Material,
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc = r.origin - self.center;
        let a = r.direction.length_squared();
        let half_b = oc.dot(r.direction);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }

        let t = root;
        let p = r.at(t);
        let outward_normal = (p - self.center) / self.radius;
        let mut rec = HitRecord {
            t,
            p,
            normal: Vec3A::ZERO, // Placeholder
            front_face: false, // Placeholder
            material: self.material.clone(),
        };
        rec.set_face_normal(r, outward_normal);

        Some(rec)
    }
}
