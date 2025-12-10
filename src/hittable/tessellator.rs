use crate::hittable::triangle::Triangle;
use crate::material::Material;
use std::collections::HashMap;

pub fn tessellate_triangle(
    triangle: &Triangle,
    subdivision_level: Option<u32>,
    max_edge_length: Option<f32>,
    material: &Material,
    displacement_strength: f32,
) -> Vec<Triangle> {
    let mut vertices = vec![triangle.v0, triangle.v0 + triangle.v0v1, triangle.v0 + triangle.v0v2];
    let mut uvs = vec![triangle.uv0, triangle.uv1, triangle.uv2];
    // We start with the original normals, but they will be re-calculated if displacement occurs.
    let mut normals = vec![triangle.n0, triangle.n1, triangle.n2];
    let mut indices = vec![[0, 1, 2]];

    const MAX_TRIANGLES: usize = 65536; // Safeguard against extreme tessellation

    if let Some(max_len) = max_edge_length {
        // Adaptive tessellation
        let mut i = 0;
        while i < indices.len() && indices.len() < MAX_TRIANGLES {
            let tri_indices = indices[i];
            let v0 = vertices[tri_indices[0]];
            let v1 = vertices[tri_indices[1]];
            let v2 = vertices[tri_indices[2]];

            let edge01_len = (v1 - v0).length();
            let edge12_len = (v2 - v1).length();
            let edge20_len = (v0 - v2).length();

            if edge01_len > max_len || edge12_len > max_len || edge20_len > max_len {
                indices.remove(i);
                subdivide_indexed(tri_indices, &mut vertices, &mut uvs, &mut normals, &mut indices);
            } else {
                i += 1;
            }
        }
    } else if let Some(sub_level) = subdivision_level {
        // Fixed subdivision
        for _ in 0..sub_level {
            if indices.len() * 4 > MAX_TRIANGLES { break; }
            let current_indices = std::mem::take(&mut indices);
            for tri_indices in current_indices {
                subdivide_indexed(tri_indices, &mut vertices, &mut uvs, &mut normals, &mut indices);
            }
        }
    }

    // Apply displacement
    if displacement_strength > 0.0 {
        if let Some(displacement_map_texture) = get_displacement_map_from_material(material) {
            for i in 0..vertices.len() {
                displace_vertex(&mut vertices[i], &mut normals[i], &uvs[i], displacement_map_texture, displacement_strength);
            }

            // Recalculate smooth normals after displacement
            let mut vertex_normals: Vec<Vec3A> = vec![Vec3A::ZERO; vertices.len()];
            let mut face_normals = Vec::new();
            for tri_indices in &indices {
                let v0 = vertices[tri_indices[0]];
                let v1 = vertices[tri_indices[1]];
                let v2 = vertices[tri_indices[2]];
                let face_normal = (v1 - v0).cross(v2 - v0).normalize_or_zero();
                face_normals.push(face_normal);
            }

            for (i, tri_indices) in indices.iter().enumerate() {
                vertex_normals[tri_indices[0]] += face_normals[i];
                vertex_normals[tri_indices[1]] += face_normals[i];
                vertex_normals[tri_indices[2]] += face_normals[i];
            }
            
            for normal in vertex_normals.iter_mut() {
                *normal = normal.normalize_or_zero();
            }
            normals = vertex_normals;
        }
    }

    // Create final triangles
    indices.into_iter().map(|tri_indices| {
        Triangle::new(
            vertices[tri_indices[0]],
            vertices[tri_indices[1]],
            vertices[tri_indices[2]],
            uvs[tri_indices[0]],
            uvs[tri_indices[1]],
            uvs[tri_indices[2]],
            normals[tri_indices[0]],
            normals[tri_indices[1]],
            normals[tri_indices[2]],
            material.clone(),
        )
    }).collect()
}

// Helper to manage indexed subdivision
fn subdivide_indexed(
    indices: [usize; 3],
    vertices: &mut Vec<Vec3A>,
    uvs: &mut Vec<Vec2>,
    normals: &mut Vec<Vec3A>,
    new_indices: &mut Vec<[usize; 3]>,
) {
    let [i0, i1, i2] = indices;
    
    // Create midpoints
    let v01 = (vertices[i0] + vertices[i1]) / 2.0;
    let v12 = (vertices[i1] + vertices[i2]) / 2.0;
    let v20 = (vertices[i2] + vertices[i0]) / 2.0;

    let uv01 = (uvs[i0] + uvs[i1]) / 2.0;
    let uv12 = (uvs[i1] + uvs[i2]) / 2.0;
    let uv20 = (uvs[i2] + uvs[i0]) / 2.0;

    let n01 = ((normals[i0] + normals[i1]) / 2.0).normalize_or_zero();
    let n12 = ((normals[i1] + normals[i2]) / 2.0).normalize_or_zero();
    let n20 = ((normals[i2] + normals[i0]) / 2.0).normalize_or_zero();

    let i01 = get_or_add_vertex(v01, uv01, n01, vertices, uvs, normals);
    let i12 = get_or_add_vertex(v12, uv12, n12, vertices, uvs, normals);
    let i20 = get_or_add_vertex(v20, uv20, n20, vertices, uvs, normals);

    new_indices.push([i0, i01, i20]);
    new_indices.push([i01, i1, i12]);
    new_indices.push([i20, i12, i2]);
    new_indices.push([i01, i12, i20]);
}

fn get_or_add_vertex(
    v: Vec3A,
    uv: Vec2,
    n: Vec3A,
    vertices: &mut Vec<Vec3A>,
    uvs: &mut Vec<Vec2>,
    normals: &mut Vec<Vec3A>,
) -> usize {
    // Simple linear search for existing vertex. For high performance, a HashMap or spatial structure would be better.
    if let Some(pos) = vertices.iter().position(|&vert| vert.abs_diff_eq(v, 1e-5)) {
        return pos;
    }
    let new_index = vertices.len();
    vertices.push(v);
    uvs.push(uv);
    normals.push(n);
    new_index
}

// Helper to get displacement map from material
fn get_displacement_map_from_material(material: &Material) -> Option<&image::Rgb32FImage> {
    match material {
        Material::Lambertian { displacement_map, .. } => {
            if let Some(tex) = displacement_map {
                if let crate::material::Texture::Image(img) = tex {
                    return Some(img);
                }
            }
        }
        Material::Metallic { displacement_map, .. } => {
            if let Some(tex) = displacement_map {
                if let crate::material::Texture::Image(img) = tex {
                    return Some(img);
                }
            }
        }
        Material::Dielectric { displacement_map, .. } => {
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

    // Note: Re-calculating the normal after displacement for smooth shading
    // might be more complex, involving sampling neighbors or using a different
    // normal mapping approach. For now, we reuse the original interpolated normal,
    // which might not be entirely accurate for displaced geometry, but
    // per-vertex normals are a start. For detailed self-shadowing,
    // this would need a more sophisticated normal recalculation (e.g.,
    // by sampling displacement map at nearby UVs and deriving a new normal).
    // Or just rely on the new sub-triangles having their own accurate normals.
}