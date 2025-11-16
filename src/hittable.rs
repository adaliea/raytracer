use std::sync::Arc;
use crate::ray::Ray;
use glam::{Vec2, Vec3A};
use crate::material::Material;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HitRecord<'a> {
    /// Point Ray intersected
    pub p: Vec3A,
    pub normal: Vec3A,
    /// Distance Ray traveled
    pub t: f32,
    pub front_face: bool,
    pub material: &'a Material,
}

impl HitRecord<'_> {
    #[inline(always)]
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
    fn hit(&'_ self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord<'_>>;
}

#[derive(Debug, Clone)]
pub enum HittableObject {
    Sphere(Sphere),
    Triangle(Triangle),
}

impl Hittable for HittableObject {
    fn hit(&'_ self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord<'_>> {
        match self {
            HittableObject::Sphere(s) => s.hit(r, t_min, t_max),
            HittableObject::Triangle(t) => t.hit(r, t_min, t_max)
        }
    }
}

impl HittableObject {
    pub fn get_material(&self) -> &Material {
        match self { HittableObject::Sphere(Sphere {material, ..}) => {material}, HittableObject::Triangle(Triangle {material, ..}) => material }
    }
}

#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Vec3A,
    pub radius: f32,
    pub material: Arc<Material>,
}

impl Hittable for Sphere {
    #[inline(always)]
    fn hit(&'_ self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord<'_>> {
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
            front_face: false,   // Placeholder
            material: &self.material,
        };
        rec.set_face_normal(r, outward_normal);

        Some(rec)
    }
}

#[derive(Debug, Clone)]
pub struct Triangle {
    pub v0: Vec3A,
    pub v1: Vec3A,
    pub v2: Vec3A,
    pub material: Arc<Material>,
    pub uv0: Vec2,
    pub uv1: Vec2,
    pub uv2: Vec2,
}


impl Hittable for Triangle {
    #[inline(always)]
    fn hit(&'_ self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord<'_>> {
        todo!()
    }
}



