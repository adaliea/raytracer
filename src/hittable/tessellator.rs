use crate::hittable::triangle::Triangle;
use crate::material::Material;
use glam::{Vec2, Vec3A};

pub fn tessellate_triangle(
    triangle: &Triangle,
    subdivision_level: u32,
    material: &Material,
    displacement_strength: f32, // New parameter for displacement strength
) -> Vec<Triangle> {
    if subdivision_level == 0 {
        let mut displaced_v0 = triangle.v0;
        let mut displaced_v1 = triangle.v0 + triangle.v0v1; // Calculate actual v1
        let mut displaced_v2 = triangle.v0 + triangle.v0v2; // Calculate actual v2

        let mut displaced_n0 = triangle.n0;
        let mut displaced_n1 = triangle.n1;
        let mut displaced_n2 = triangle.n2;

        if let Some(displacement_map_texture) = get_displacement_map_from_material(material) {
            displace_vertex(
                &mut displaced_v0,
                &mut displaced_n0,
                &triangle.uv0,
                displacement_map_texture,
                displacement_strength,
            );
            displace_vertex(
                &mut displaced_v1,
                &mut displaced_n1,
                &triangle.uv1,
                displacement_map_texture,
                displacement_strength,
            );
            displace_vertex(
                &mut displaced_v2,
                &mut displaced_n2,
                &triangle.uv2,
                displacement_map_texture,
                displacement_strength,
            );
        }
        // Now, construct a NEW Triangle with these displaced vertices and normals.
        // The new Triangle::new constructor will re-calculate v0v1 and v0v2.
        return vec![Triangle::new(
            displaced_v0,
            displaced_v1,
            displaced_v2,
            triangle.uv0,
            triangle.uv1,
            triangle.uv2,
            displaced_n0,
            displaced_n1,
            displaced_n2,
            material.clone(),
        )];
    }

    let mut subdivided_triangles = Vec::new();

    // Vertices of the original triangle
    let v0 = triangle.v0;
    let v1 = triangle.v0 + triangle.v0v1;
    let v2 = triangle.v0 + triangle.v0v2;

    // UVs
    let uv0 = triangle.uv0;
    let uv1 = triangle.uv1;
    let uv2 = triangle.uv2;

    // Normals
    let n0 = triangle.n0;
    let n1 = triangle.n1;
    let n2 = triangle.n2;

    // Midpoints
    let v01 = (v0 + v1) / 2.0;
    let v12 = (v1 + v2) / 2.0;
    let v20 = (v2 + v0) / 2.0;

    // Midpoint UVs
    let uv01 = (uv0 + uv1) / 2.0;
    let uv12 = (uv1 + uv2) / 2.0;
    let uv20 = (uv2 + uv0) / 2.0;

    // Midpoint Normals (linear interpolation and re-normalize)
    let n01 = ((n0 + n1) / 2.0).normalize();
    let n12 = ((n1 + n2) / 2.0).normalize();
    let n20 = ((n2 + n0) / 2.0).normalize();

    // Create 4 new triangles
    let t1 = Triangle::new(v0, v01, v20, uv0, uv01, uv20, n0, n01, n20, material.clone());
    let t2 = Triangle::new(v01, v1, v12, uv01, uv1, uv12, n01, n1, n12, material.clone());
    let t3 = Triangle::new(v20, v12, v2, uv20, uv12, uv2, n20, n12, n2, material.clone());
    let t4 = Triangle::new(v01, v12, v20, uv01, uv12, uv20, n01, n12, n20, material.clone());

    // Recursively tessellate
    subdivided_triangles.extend(tessellate_triangle(&t1, subdivision_level - 1, material, displacement_strength));
    subdivided_triangles.extend(tessellate_triangle(&t2, subdivision_level - 1, material, displacement_strength));
    subdivided_triangles.extend(tessellate_triangle(&t3, subdivision_level - 1, material, displacement_strength));
    subdivided_triangles.extend(tessellate_triangle(&t4, subdivision_level - 1, material, displacement_strength));

    subdivided_triangles
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