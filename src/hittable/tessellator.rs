use crate::hittable::triangle::Triangle;
use crate::material::Material;
use glam::{Vec2, Vec3A};
use log::warn;

pub fn tessellate_triangle(
    triangle: &Triangle,
    subdivision_level: Option<u32>,
    max_edge_length: Option<f32>,
    material: &Material,
    displacement_strength: f32,
) -> Vec<Triangle> {
    let mut triangles = vec![triangle.clone()];

    if let Some(max_len) = max_edge_length {
        // Adaptive tessellation
        let mut i = 0;
        while i < triangles.len() {
            let tri = &triangles[i];
            let v0 = tri.v0;
            let v1 = tri.v0 + tri.v0v1;
            let v2 = tri.v0 + tri.v0v2;

            let edge01_len = (v1 - v0).length();
            let edge12_len = (v2 - v1).length();
            let edge20_len = (v0 - v2).length();

            if edge01_len > max_len || edge12_len > max_len || edge20_len > max_len {
                let subdivided = subdivide_one_triangle(tri, material);
                triangles.splice(i..=i, subdivided);
            } else {
                i += 1;
            }
        }
    } else if let Some(sub_level) = subdivision_level {
        for _ in 0..sub_level {
            let mut next_triangles = Vec::new();
            for tri in triangles {
                next_triangles.extend(subdivide_one_triangle(&tri, material));
            }
            triangles = next_triangles;
        }
    }

    // Apply displacement
    if displacement_strength > 0.0 {
        if let Some(displacement_map_texture) = get_displacement_map_from_material(material) {
            for tri in &mut triangles {
                let mut v0 = tri.v0;
                let mut v1 = tri.v0 + tri.v0v1;
                let mut v2 = tri.v0 + tri.v0v2;

                let mut n0 = tri.n0;
                let mut n1 = tri.n1;
                let mut n2 = tri.n2;

                displace_vertex(
                    &mut v0,
                    &mut n0,
                    &tri.uv0,
                    displacement_map_texture,
                    displacement_strength,
                );
                displace_vertex(
                    &mut v1,
                    &mut n1,
                    &tri.uv1,
                    displacement_map_texture,
                    displacement_strength,
                );
                displace_vertex(
                    &mut v2,
                    &mut n2,
                    &tri.uv2,
                    displacement_map_texture,
                    displacement_strength,
                );

                // Don't use new_tri.normal. Use n0, n1, n2.
                *tri = Triangle::new(
                    v0,
                    v1,
                    v2,
                    tri.uv0,
                    tri.uv1,
                    tri.uv2,
                    n0,
                    n1,
                    n2,
                    material.clone(),
                );
            }
        }
    }

    triangles
}

fn subdivide_one_triangle(tri: &Triangle, material: &Material) -> Vec<Triangle> {
    let v0 = tri.v0;
    let v1 = tri.v0 + tri.v0v1;
    let v2 = tri.v0 + tri.v0v2;

    let uv0 = tri.uv0;
    let uv1 = tri.uv1;
    let uv2 = tri.uv2;

    let n0 = tri.n0;
    let n1 = tri.n1;
    let n2 = tri.n2;

    let v01 = (v0 + v1) / 2.0;
    let v12 = (v1 + v2) / 2.0;
    let v20 = (v2 + v0) / 2.0;

    let uv01 = (uv0 + uv1) / 2.0;
    let uv12 = (uv1 + uv2) / 2.0;
    let uv20 = (uv2 + uv0) / 2.0;

    let n01 = ((n0 + n1) / 2.0).normalize_or_zero();
    let n12 = ((n1 + n2) / 2.0).normalize_or_zero();
    let n20 = ((n2 + n0) / 2.0).normalize_or_zero();

    vec![
        Triangle::new(
            v0,
            v01,
            v20,
            uv0,
            uv01,
            uv20,
            n0,
            n01,
            n20,
            material.clone(),
        ),
        Triangle::new(
            v01,
            v1,
            v12,
            uv01,
            uv1,
            uv12,
            n01,
            n1,
            n12,
            material.clone(),
        ),
        Triangle::new(
            v20,
            v12,
            v2,
            uv20,
            uv12,
            uv2,
            n20,
            n12,
            n2,
            material.clone(),
        ),
        Triangle::new(
            v01,
            v12,
            v20,
            uv01,
            uv12,
            uv20,
            n01,
            n12,
            n20,
            material.clone(),
        ),
    ]
}

// Helper to get displacement map from material
fn get_displacement_map_from_material(material: &Material) -> Option<&image::Rgb32FImage> {
    match material {
        Material::Lambertian {
            displacement_map, ..
        } => {
            if let Some(tex) = displacement_map {
                if let crate::material::Texture::Image(img) = tex {
                    return Some(img);
                }
            }
        }
        Material::Metallic {
            displacement_map, ..
        } => {
            if let Some(tex) = displacement_map {
                if let crate::material::Texture::Image(img) = tex {
                    return Some(img);
                }
            }
        }
        Material::Dielectric {
            displacement_map, ..
        } => {
            if let Some(tex) = displacement_map {
                if let crate::material::Texture::Image(img) = tex {
                    return Some(img);
                }
            }
        }
        _ => {} // Emissive does not have displacement map
    }
    None
}

// Helper function to displace a vertex
fn displace_vertex(
    vertex: &mut Vec3A,
    normal: &mut Vec3A,
    uv: &Vec2,
    displacement_map: &image::Rgb32FImage,
    displacement_strength: f32,
) {
    let u = uv.x.rem_euclid(1.0);
    let v = 1.0 - uv.y.rem_euclid(1.0); // Flip v for image coords

    let x_img = (u * (displacement_map.width() - 1) as f32) as u32;
    let y_img = (v * (displacement_map.height() - 1) as f32) as u32;

    let pixel = displacement_map.get_pixel(x_img, y_img);
    // Assuming displacement map is grayscale, so take one channel (e.g., red)
    let displacement_value = pixel[0];

    // Displace the vertex along its normal
    *vertex += *normal * displacement_value * displacement_strength;
}
