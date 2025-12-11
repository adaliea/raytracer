use crate::hittable::{HitRecord, Hittable, HittableObject, LazyUv};
use crate::material::Material;
use crate::ray::Ray;
use crate::scene::Scene;
use bvh::bounding_hierarchy::BHShape;
use glam::{Mat3A, Vec2, Vec3A};
use rand::Rng;
use rand_distr::{Distribution, UnitSphere};

pub struct TraceResult {
    pub color: Vec3A,
    pub albedo: Vec3A,
    pub normal: Vec3A,
    /// How many ray intersection tests occurred
    pub rays: u64,
}
#[inline(always)]
pub fn ray_color(
    r: &Ray,
    world: &Scene,
    depth: u32,
    max_bounces: u32,
    is_specular_ray: bool,
    rng: &mut impl Rng,
) -> TraceResult {
    if depth == 0 {
        return TraceResult {
            color: Vec3A::ZERO,
            albedo: Vec3A::ZERO,
            normal: Vec3A::ZERO,
            rays: 0,
        };
    }

    if let Some(rec) = world.hit(r, 0.001, f32::INFINITY) {
        return match rec.material {
            Material::Emissive {
                emit_color,
                strength,
            } => {
                let emit = emit_color.sample(&rec.uv) * *strength;
                return if depth == max_bounces || is_specular_ray {
                    TraceResult {
                        color: emit,
                        albedo: Vec3A::ZERO, // Emissive surfaces have no albedo
                        normal: Vec3A::ZERO, // Or normal
                        rays: 1,
                    } // It's a camera ray, return the light
                } else {
                    TraceResult {
                        color: Vec3A::ZERO,
                        albedo: Vec3A::ZERO,
                        normal: Vec3A::ZERO,
                        rays: 1,
                    } // It's an indirect bounce, don't double-count.
                };
            }

            Material::Lambertian { albedo, normal_map, .. } => {
                let attenuation = albedo.sample(&rec.uv);
                let shading_normal = get_perturbed_normal(normal_map, &rec);

                let direct_trace = sample_direct_light(world, &rec, shading_normal, attenuation, rng);

                // rr to for GI bounces
                let probability = (attenuation.max_element().max(0.01) * 2.0).min(1.0);
                if depth < (max_bounces - 5) && rand::random::<f32>() > probability {
                    return TraceResult {
                        color: direct_trace.color,
                        albedo: attenuation,
                        normal: shading_normal,
                        rays: direct_trace.rays,
                    };
                }

                let scatter_direction = shading_normal + random_on_unit_sphere(rng);

                let scattered_ray = Ray::new(rec.p, scatter_direction);

                let indirect_trace =
                    ray_color(&scattered_ray, world, depth - 1, max_bounces, false, rng);
                // Calculate indirect light
                let indirect_light = (attenuation * indirect_trace.color) / probability;

                TraceResult {
                    color: direct_trace.color + indirect_light,
                    albedo: attenuation,
                    normal: shading_normal,
                    rays: indirect_trace.rays + direct_trace.rays + 1,
                }
            }

            Material::Metallic {
                albedo,
                fuzz,
                normal_map,
                ..
            } => {
                let attenuation = albedo.sample(&rec.uv);
                let shading_normal = get_perturbed_normal(normal_map, &rec);

                let reflected_direction = reflect(r.direction.normalize(), shading_normal);
                let scattered_ray = Ray::new(
                    rec.p,
                    reflected_direction + fuzz * random_in_unit_sphere(rng),
                );

                let reflected_trace =
                    ray_color(&scattered_ray, world, depth - 1, max_bounces, true, rng);
                if scattered_ray.direction.dot(shading_normal) > 0.0 {
                    TraceResult {
                        color: attenuation * reflected_trace.color,
                        albedo: attenuation,
                        normal: shading_normal,
                        rays: reflected_trace.rays + 1,
                    }
                } else {
                    TraceResult {
                        color: Vec3A::ZERO,
                        albedo: attenuation,
                        normal: shading_normal,
                        rays: reflected_trace.rays + 1,
                    }
                }
            }

            Material::Dielectric {
                index_of_refraction,
                fuzz,
                ..
            } => {
                let attenuation = Vec3A::ONE; // Dielectrics are white
                // Dielectrics do not use normal mapping in this implementation
                // (it's complex to get right with refraction)
                let shading_normal = rec.normal;

                let refraction_ratio = if rec.front_face {
                    1.0 / index_of_refraction
                } else {
                    *index_of_refraction
                };

                let unit_direction = r.direction.normalize();
                let cos_theta = (-unit_direction).dot(shading_normal).min(1.0);
                let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
                let cannot_refract = refraction_ratio * sin_theta > 1.0;

                let reflectance = schlick(cos_theta, refraction_ratio);

                let scatter_direction = if cannot_refract || reflectance > rand::random::<f32>() {
                    reflect(unit_direction, shading_normal)
                } else {
                    refract(
                        unit_direction,
                        shading_normal + random_on_unit_sphere(rng) * fuzz,
                        refraction_ratio,
                    )
                };

                let scattered_ray = Ray::new(rec.p, scatter_direction);
                let scattered_trace =
                    ray_color(&scattered_ray, world, depth - 1, max_bounces, true, rng);
                TraceResult {
                    color: attenuation * scattered_trace.color,
                    albedo: attenuation,
                    normal: shading_normal,
                    rays: scattered_trace.rays + 1,
                }
            }
        };
    }

    TraceResult {
        color: world.background_color,
        albedo: Vec3A::ZERO,
        normal: Vec3A::ZERO,
        rays: 1,
    }
}

/// Samples all lights for a given hit point (NEE)
#[inline(always)]
fn sample_direct_light(
    world: &Scene,
    rec: &HitRecord,
    shading_normal: Vec3A,
    attenuation: Vec3A,
    rng: &mut impl Rng,
) -> TraceResult {
    let mut total_direct_light = Vec3A::ZERO;

    let num_shadow_samples = 2; // higher = slower, but better quality
    let total_samples = world.lights.len() * num_shadow_samples;

    if total_samples == 0 {
        return TraceResult {
            color: Vec3A::ZERO,
            albedo: Vec3A::ZERO,
            normal: Vec3A::ZERO,
            rays: 0,
        };
    }

    for light_index in &world.lights {
        let light_obj = &world.objects[*light_index];

        if let HittableObject::Sphere(light_sphere) = light_obj {
            // Get the light's emission
            let (light_emit, light_strength) = match &light_sphere.material {
                Material::Emissive {
                    emit_color,
                    strength,
                } => {
                    (emit_color.sample(&LazyUv::Uv(Vec2::ZERO)), strength) // UVs don't matter for solid color
                }
                _ => continue, // Not an emissive material
            };
            let emitted_light = light_emit * light_strength;

            let mut light_contribution = Vec3A::ZERO;

            // Cast multiple shadow rays for soft shadows
            for _ in 0..num_shadow_samples {
                // Pick a random point on the light sphere on the side facing the hit point
                let rand_dir = random_on_unit_sphere(rng);
                let light_point = light_sphere.center + rand_dir * light_sphere.radius;

                let shadow_dir = light_point - rec.p;
                let shadow_dist_2 = shadow_dir.length_squared();
                let shadow_ray = Ray::new(rec.p, shadow_dir);

                // Check if the ray is occluded
                let shadow_ray_rec = world.hit(&shadow_ray, 0.00001, f32::INFINITY);

                if shadow_ray_rec
                    .map(|r| r.bh_object_index == light_sphere.bh_node_index())
                    .unwrap_or(true)
                {
                    let cos_theta = shading_normal.dot(shadow_dir.normalize()).max(0.0);

                    // (1/dist^2) falloff

                    // (albedo * light * cos_theta) / dist^2
                    light_contribution += (attenuation * emitted_light * cos_theta) / shadow_dist_2;
                }
            }

            total_direct_light += light_contribution / num_shadow_samples as f32;
        }
    }

    TraceResult {
        color: total_direct_light,
        albedo: Vec3A::ZERO,
        normal: Vec3A::ZERO,
        rays: total_samples as u64,
    }
}

/// Generates a random 3D vector uniformly INSIDE a unit sphere.
#[inline(always)]
fn random_in_unit_sphere(rng: &mut impl Rng) -> Vec3A {
    loop {
        // Generate a vector in the cube [-1, 1]
        let v = Vec3A::new(
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
        );

        // If it is inside the sphere, return it.
        if v.length_squared() < 1.0 {
            return v;
        }
    }
}

/// Generates a random 3D vector uniformly ON a unit sphere surface.
#[inline(always)]
fn random_on_unit_sphere(rng: &mut impl Rng) -> Vec3A {
    let [x, y, z]: [f32; 3] = UnitSphere.sample(rng);
    Vec3A::new(x, y, z)
}

/// Reflects an incoming vector `v` off a surface with normal `n`
#[inline(always)]
fn reflect(v: Vec3A, n: Vec3A) -> Vec3A {
    v - 2.0 * v.dot(n) * n
}

/// Refracts an incoming vector `uv` (unit vector) at a surface `n`
/// with a refraction ratio `etai_over_etat`
#[inline(always)]
fn refract(uv: Vec3A, n: Vec3A, etai_over_etat: f32) -> Vec3A {
    let cos_theta = (-uv).dot(n).min(1.0);
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = -(1.0 - r_out_perp.length_squared()).abs().sqrt() * n;
    r_out_perp + r_out_parallel
}

/// Schlick's approximation for reflectance
#[inline(always)]
fn schlick(cosine: f32, ref_ratio: f32) -> f32 {
    let r0 = (1.0 - ref_ratio) / (1.0 + ref_ratio);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

#[inline(always)]
fn get_perturbed_normal(
    normal_map: &Option<crate::material::Texture>,
    rec: &HitRecord,
) -> Vec3A {
    if let Some(map) = normal_map {
        let tangent_normal = map.sample(&rec.uv) * 2.0 - 1.0;
        let tbn = Mat3A::from_cols(rec.tangent, rec.bitangent, rec.normal);
        (tbn * tangent_normal).normalize()
    } else {
        rec.normal
    }
}
