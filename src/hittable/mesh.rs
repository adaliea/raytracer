use crate::hittable::{HitRecord, Hittable};
use crate::hittable::triangle::Triangle;
use crate::material::Material;
use crate::ray::Ray;
use bvh::aabb::{Aabb, Bounded};
use bvh::bounding_hierarchy::{BHShape, BoundingHierarchy};
use bvh::bvh::Bvh;
use nalgebra::Point3;
use glam::{Vec3A, Vec2};
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct Mesh {
    triangles: Vec<Triangle>,
    bvh: Bvh<f32, 3>,
    node_index: usize,
}

impl Mesh {
    pub fn new(triangles: Vec<Triangle>) -> Self {
        let mut bvh_shapes = triangles.clone();
        let bvh = Bvh::build_par(&mut bvh_shapes);
        Self {
            triangles,
            bvh,
            node_index: 0,
        }
    }

    pub fn generate_cylinder_triangles(
        center: Vec3A,
        radius: f32,
        height: f32,
        segments: usize,
        material: Material,
    ) -> Vec<Triangle> {
        let mut triangles = Vec::new();
        let half_height = height / 2.0;

        // Top and bottom centers
        let top_center = center + Vec3A::new(0.0, half_height, 0.0);
        let bottom_center = center - Vec3A::new(0.0, half_height, 0.0);

        for i in 0..segments {
            let theta0 = 2.0 * PI * (i as f32) / (segments as f32);
            let theta1 = 2.0 * PI * ((i + 1) as f32) / (segments as f32);

            let x0 = radius * theta0.cos();
            let z0 = radius * theta0.sin();
            let x1 = radius * theta1.cos();
            let z1 = radius * theta1.sin();

            // Vertices for the current segment
            let v0_bottom = bottom_center + Vec3A::new(x0, 0.0, z0);
            let v1_bottom = bottom_center + Vec3A::new(x1, 0.0, z1);
            let v0_top = top_center + Vec3A::new(x0, 0.0, z0);
            let v1_top = top_center + Vec3A::new(x1, 0.0, z1);

            // UVs (cylindrical mapping)
            let uv_x0 = i as f32 / segments as f32;
            let uv_x1 = (i + 1) as f32 / segments as f32;
            let uv_y_bottom = 0.0;
            let uv_y_top = 1.0;

            let uv00 = Vec2::new(uv_x0, uv_y_bottom);
            let uv01 = Vec2::new(uv_x1, uv_y_bottom);
            let uv10 = Vec2::new(uv_x0, uv_y_top);
            let uv11 = Vec2::new(uv_x1, uv_y_top);

            // Side surface (two triangles per segment)
            // Triangle 1: v0_bottom, v1_top, v0_top
            let normal_side1 = (v1_top - v0_bottom).cross(v0_top - v0_bottom).normalize();
            triangles.push(Triangle::new(
                v0_bottom,
                v1_top,
                v0_top,
                uv00,
                uv11,
                uv10,
                normal_side1,
                normal_side1,
                normal_side1, // Per-vertex normals for now are just face normal
                material.clone(),
            ));

            // Triangle 2: v0_bottom, v1_bottom, v1_top
            let normal_side2 = (v1_bottom - v0_bottom).cross(v1_top - v0_bottom).normalize();
            triangles.push(Triangle::new(
                v0_bottom,
                v1_bottom,
                v1_top,
                uv00,
                uv01,
                uv11,
                normal_side2,
                normal_side2,
                normal_side2, // Per-vertex normals for now are just face normal
                material.clone(),
            ));

            // Top cap
            let normal_top = Vec3A::new(0.0, 1.0, 0.0); // Normal pointing up
            triangles.push(Triangle::new(
                top_center,
                v1_top,
                v0_top,
                Vec2::new(0.5, 0.5),
                Vec2::new(x1 / (2.0 * radius) + 0.5, z1 / (2.0 * radius) + 0.5),
                Vec2::new(x0 / (2.0 * radius) + 0.5, z0 / (2.0 * radius) + 0.5), // Planar mapping for cap
                normal_top,
                normal_top,
                normal_top,
                material.clone(),
            ));

            // Bottom cap
            let normal_bottom = Vec3A::new(0.0, -1.0, 0.0); // Normal pointing down
            triangles.push(Triangle::new(
                bottom_center,
                v0_bottom,
                v1_bottom,
                Vec2::new(0.5, 0.5),
                Vec2::new(x0 / (2.0 * radius) + 0.5, z0 / (2.0 * radius) + 0.5),
                Vec2::new(x1 / (2.0 * radius) + 0.5, z1 / (2.0 * radius) + 0.5), // Planar mapping for cap
                normal_bottom,
                normal_bottom,
                normal_bottom,
                material.clone(),
            ));
        }

        triangles
    }
}

impl Hittable for Mesh {
    fn hit(&'_ self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord<'_>> {
        let mut closest_so_far = t_max;
        let mut hit_anything = None;

        let bvh_ray = bvh::ray::Ray::new(
            Point3::new(r.origin.x, r.origin.y, r.origin.z),
            nalgebra::Vector3::new(r.direction.x, r.direction.y, r.direction.z),
        );
        let hit_objects = self.bvh.traverse(&bvh_ray, &self.triangles);

        for object in hit_objects {
            if let Some(rec) = object.hit(r, t_min, closest_so_far) {
                closest_so_far = rec.t;
                hit_anything = Some(rec);
            }
        }
        hit_anything
    }
}

impl Bounded<f32, 3> for Mesh {
    fn aabb(&self) -> Aabb<f32, 3> {
        let mut min_point = Point3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max_point = Point3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);

        for triangle in &self.triangles {
            let aabb = triangle.aabb();
            min_point.x = min_point.x.min(aabb.min.x);
            min_point.y = min_point.y.min(aabb.min.y);
            min_point.z = min_point.z.min(aabb.min.z);

            max_point.x = max_point.x.max(aabb.max.x);
            max_point.y = max_point.y.max(aabb.max.y);
            max_point.z = max_point.z.max(aabb.max.z);
        }

        Aabb::with_bounds(min_point, max_point)
    }
}

impl BHShape<f32, 3> for Mesh {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}
