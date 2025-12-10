use crate::hittable::LazyUv::Uv;
use crate::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use bvh::aabb::{Aabb, Bounded};
use bvh::bounding_hierarchy::BHShape;
use glam::{Vec2, Vec3A};
use nalgebra::Point3;

#[derive(Debug, Clone)]
pub struct Triangle {
    pub v0: Vec3A,
    pub material: Material,
    pub uv0: Vec2,
    pub uv1: Vec2,
    pub uv2: Vec2,
    pub n0: Vec3A,
    pub n1: Vec3A,
    pub n2: Vec3A,
    pub v0v1: Vec3A,
    pub v0v2: Vec3A,
    pub normal: Vec3A,
    pub tangent: Vec3A,
    pub bitangent: Vec3A,
    pub d: f32,
    pub dot00: f32,
    pub dot01: f32,
    pub dot11: f32,
    pub inv_denom: f32,
    node_index: usize,
}

impl Triangle {
    pub fn new(
        v0: Vec3A,
        v1: Vec3A,
        v2: Vec3A,
        uv0: Vec2,
        uv1: Vec2,
        uv2: Vec2,
        n0: Vec3A,
        n1: Vec3A,
        n2: Vec3A,
        material: Material,
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

        let delta_uv1 = uv1 - uv0;
        let delta_uv2 = uv2 - uv0;

        let f = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);

        let tangent = (f * (delta_uv2.y * v0v1 - delta_uv1.y * v0v2)).normalize();
        let bitangent = (f * (-delta_uv2.x * v0v1 + delta_uv1.x * v0v2)).normalize();

        Self {
            v0,
            uv0,
            uv1,
            uv2,
            material,
            n0,
            n1,
            n2,
            v0v1,
            v0v2,
            normal,
            tangent,
            bitangent,
            d,
            dot00,
            dot01,
            dot11,
            inv_denom,
            node_index: 0,
        }
    }
}

impl Hittable for Triangle {
    #[inline(always)]
    fn hit(&'_ self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord<'_>> {
        // Find Plane Intersection
        let dot_normal_dir = self.normal.dot(r.direction);
        if dot_normal_dir == 0.0 {
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
        let w = 1.0 - u - v;
        let interpolated_normal = (self.n0 * w + self.n1 * u + self.n2 * v).normalize();

        let uv = self.uv0 * w + self.uv1 * u + self.uv2 * v;

        let mut rec = HitRecord {
            t,
            p,
            normal: Vec3A::ZERO,
            tangent: self.tangent,
            bitangent: self.bitangent,
            front_face: false,
            material: &self.material,
            uv: Uv(uv),
            bh_object_index: self.node_index,
        };
        rec.set_face_normal(r, interpolated_normal);

        Some(rec)
    }
}

impl Bounded<f32, 3> for Triangle {
    #[inline(always)]

    fn aabb(&self) -> Aabb<f32, 3> {
        let mut min = Point3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = Point3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);

        for v in [self.v0, self.v0 + self.v0v1, self.v0 + self.v0v2] {
            if v.x < min.x {
                min.x = v.x
            }
            if v.y < min.y {
                min.y = v.y
            }
            if v.z < min.z {
                min.z = v.z
            }
            if v.x > max.x {
                max.x = v.x
            }
            if v.y > max.y {
                max.y = v.y
            }
            if v.z > max.z {
                max.z = v.z
            }
        }
        Aabb::with_bounds(min, max)
    }
}

impl BHShape<f32, 3> for Triangle {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }
    #[inline(always)]
    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}
