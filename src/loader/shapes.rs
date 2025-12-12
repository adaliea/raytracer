use crate::hittable::triangle::Triangle;
use crate::material::Material;
use glam::{Vec2, Vec3A};
use std::f32::consts::PI;

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
        // Calculate smooth normals for the vertices on the cylinder side
        let normal_v0 = (v0_top - top_center).normalize();
        let normal_v1 = (v1_top - top_center).normalize();

        // Triangle 1: (v0_bottom, v0_top, v1_top)
        triangles.push(Triangle::new(
            v0_bottom,
            v0_top,
            v1_top,
            uv00,
            uv10,
            uv11,
            normal_v0,
            normal_v0,
            normal_v1,
            material.clone(),
        ));

        // Triangle 2: (v0_bottom, v1_top, v1_bottom)
        triangles.push(Triangle::new(
            v0_bottom,
            v1_top,
            v1_bottom,
            uv00,
            uv11,
            uv01,
            normal_v0,
            normal_v1,
            normal_v1,
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
