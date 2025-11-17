use crate::material::Material;
use crate::ray::Ray;
use glam::{Vec2, Vec3A};
use std::f32::consts::PI;
use std::sync::Arc;

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
    pub uv: Vec2,
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
            HittableObject::Triangle(t) => t.hit(r, t_min, t_max),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Vec3A,
    pub radius: f32,
    pub material: Arc<Material>,
}

impl Sphere {
    pub fn new(center: Vec3A, radius: f32, material: Arc<Material>) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
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

        let theta = (-outward_normal.y).acos(); // acos(-y)
        let phi = (-outward_normal.z).atan2(outward_normal.x) + PI; // atan2(-z, x) + PI

        let uv = Vec2::new(
            phi / (2.0 * PI), // u = phi / 2PI
            theta / PI,       // v = theta / PI
        );

        let mut rec = HitRecord {
            t,
            p,
            normal: Vec3A::ZERO, // Placeholder
            front_face: false,   // Placeholder
            material: &self.material,
            uv,
        };
        rec.set_face_normal(r, outward_normal);

        Some(rec)
    }
}

#[derive(Debug, Clone)]
pub struct Triangle {
    v0: Vec3A,
    material: Arc<Material>,
    uv0: Vec2,
    uv1: Vec2,
    uv2: Vec2,
    v0v1: Vec3A,
    v0v2: Vec3A,
    normal: Vec3A,
    d: f32,
    dot00: f32,
    dot01: f32,
    dot11: f32,
    inv_denom: f32,
}

impl Triangle {
    pub fn new(
        v0: Vec3A,
        v1: Vec3A,
        v2: Vec3A,
        uv0: Vec2,
        uv1: Vec2,
        uv2: Vec2,
        material: Arc<Material>,
    ) -> Self {
        let v0v1 = v1 - v0;
        let v0v2 = v2 - v0;
        let normal = v0v1.cross(v0v2).normalize();
        let d = -normal.dot(v0);

        let dot00 = v0v1.dot(v0v1);
        let dot01 = v0v1.dot(v0v2);
        let dot11 = v0v2.dot(v0v2);
        let denom = dot00 * dot11 - dot01 * dot01;
        let inv_denom = if denom.abs() < 1e-6 { 0.0 } else { 1.0 / denom };

        Self {
            v0,
            uv0,
            uv1,
            uv2,
            material,
            v0v1,
            v0v2,
            normal,
            d,
            dot00,
            dot01,
            dot11,
            inv_denom,
        }
    }
}

impl Hittable for Triangle {
    #[inline(always)]
    fn hit(&'_ self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord<'_>> {
        // Find Plane Intersection
        let dot_normal_dir = self.normal.dot(r.direction);
        if approx::abs_diff_eq!(dot_normal_dir, 0.0) {
            return None;
        }
        let t = -(self.normal.dot(r.origin) + self.d) / dot_normal_dir;
        if (t < t_min) || (t > t_max) {
            return None;
        }
        let p = r.at(t);

        // Check if Point is Inside
        if self.inv_denom == 0.0 {
            return None;
        } // Degenerate triangle

        let v0p = p - self.v0;
        let dot02 = v0p.dot(self.v0v1);
        let dot12 = v0p.dot(self.v0v2);

        // Calculate barycentric coordinates u and v
        let u = (self.dot11 * dot02 - self.dot01 * dot12) * self.inv_denom;
        let v = (self.dot00 * dot12 - self.dot01 * dot02) * self.inv_denom;

        // Check if the point is inside the triangle
        if (u < 0.0) || (v < 0.0) || (u + v > 1.0) {
            return None;
        }

        // valid hit
        let uv = self.uv0 * (1.0 - u - v) + self.uv1 * u + self.uv2 * v;
        let mut rec = HitRecord {
            t,
            p,
            normal: Vec3A::ZERO,
            front_face: false,
            material: &self.material,
            uv,
        };
        rec.set_face_normal(r, self.normal);

        Some(rec)
    }
}
