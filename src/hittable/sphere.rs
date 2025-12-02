use crate::hittable::LazyUv::LazySphere;
use crate::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use bvh::aabb::{Aabb, Bounded};
use bvh::bounding_hierarchy::BHShape;
use glam::{Vec2, Vec3A};
use nalgebra::{Point3, Vector3};
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Vec3A,
    pub radius: f32,
    pub material: Material,
    node_index: usize,
}

#[inline(always)]
pub fn calc_uv(outward_normal: &Vec3A) -> Vec2 {
    let theta = (-outward_normal.y).acos(); // acos(-y)
    let phi = (-outward_normal.z).atan2(outward_normal.x) + PI; // atan2(-z, x) + PI

    Vec2::new(
        phi / (2.0 * PI), // u = phi / 2PI
        theta / PI,       // v = theta / PI
    )
}

impl Sphere {
    pub fn new(center: Vec3A, radius: f32, material: Material) -> Self {
        Self {
            center,
            radius,
            material,
            node_index: 0,
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

        let mut rec = HitRecord {
            t,
            p,
            normal: Vec3A::ZERO, // Placeholder
            front_face: false,   // Placeholder
            material: &self.material,
            uv: LazySphere { outward_normal },
            bh_object_index: self.node_index,
        };
        rec.set_face_normal(r, outward_normal);

        Some(rec)
    }
}

impl Bounded<f32, 3> for Sphere {
    #[inline(always)]
    fn aabb(&self) -> Aabb<f32, 3> {
        let center = Point3::new(self.center.x, self.center.y, self.center.z);
        let half_size = Vector3::new(self.radius, self.radius, self.radius);
        let min = center - half_size;
        let max = center + half_size;
        Aabb::with_bounds(min, max)
    }
}

impl BHShape<f32, 3> for Sphere {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    #[inline(always)]
    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}
