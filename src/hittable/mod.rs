use crate::hittable::sphere::Sphere;
use crate::material::Material;
use crate::ray::Ray;
use bvh::aabb::Bounded;
use bvh::bounding_hierarchy::BHShape;
use glam::{Vec2, Vec3A};
use std::fmt::Debug;
use triangle::Triangle;

pub mod sphere;
pub mod triangle;

#[allow(dead_code)]
pub struct HitRecord<'a> {
    /// Point Ray intersected
    pub p: Vec3A,
    pub normal: Vec3A,
    /// Distance Ray traveled
    pub t: f32,
    pub front_face: bool,
    pub material: &'a Material,
    pub uv: LazyUv,
    pub bh_object_index: usize,
}

#[derive(Debug, Copy, Clone)]
pub enum LazyUv {
    Uv(Vec2),
    LazySphere { outward_normal: Vec3A },
}

impl LazyUv {
    pub fn get_uv(&self) -> Vec2 {
        match self {
            LazyUv::Uv(uv) => *uv,
            LazyUv::LazySphere { outward_normal } => sphere::calc_uv(outward_normal),
        }
    }
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
    #[inline(always)]
    fn hit(&'_ self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord<'_>> {
        match self {
            HittableObject::Sphere(s) => s.hit(r, t_min, t_max),
            HittableObject::Triangle(t) => t.hit(r, t_min, t_max),
        }
    }
}

impl Bounded<f32, 3> for HittableObject {
    fn aabb(&self) -> bvh::aabb::Aabb<f32, 3> {
        match self {
            HittableObject::Sphere(s) => s.aabb(),
            HittableObject::Triangle(t) => t.aabb(),
        }
    }
}

impl BHShape<f32, 3> for HittableObject {
    fn set_bh_node_index(&mut self, index: usize) {
        match self {
            HittableObject::Sphere(s) => s.set_bh_node_index(index),
            HittableObject::Triangle(t) => t.set_bh_node_index(index),
        }
    }
    fn bh_node_index(&self) -> usize {
        match self {
            HittableObject::Sphere(s) => s.bh_node_index(),
            HittableObject::Triangle(t) => t.bh_node_index(),
        }
    }
}
