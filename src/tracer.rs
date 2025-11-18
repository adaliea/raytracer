use crate::hittable::Hittable;
use crate::material::Material;
use crate::ray::Ray;
use crate::scene::Scene;
use glam::Vec3A;
use rand::Rng;
use std::cmp::max;

#[inline(always)]
pub fn ray_color(r: &Ray, world: &Scene, depth: u32, max_bounces: u32) -> Vec3A {
    if depth <= 0 {
        return Vec3A::ZERO;
    }

    if let Some(rec) = world.hit(r, 0.001, f32::INFINITY) {
        if let Material::Emissive {
            emit_color,
            strength,
        } = rec.material
        {
            return emit_color.sample(rec.uv) * *strength;
        }

        // Kill rays that are low in energy
        let attenuation = match rec.material {
            Material::Lambertian { albedo } => albedo.sample(rec.uv),
            Material::Metallic { albedo, .. } => albedo.sample(rec.uv),
            Material::Dielectric { .. } => Vec3A::ONE,
            Material::Emissive { .. } => Vec3A::ZERO,
        };

        // Pick our probability (P)
        let probability = (attenuation.max_element().max(0.01) * 2.0).min(1.0);

        // play RR with the rays
        // kill the ray if the probability is less than P
        // we compensate for this by increasing the contribution of the rays that don't die
        // to keep the energy of the scene constant
        if depth < (max_bounces - 5) {
            if rand::random::<f32>() > probability {
                return Vec3A::ZERO; // Terminate this path
            }
        }

        return match rec.material {
            // light
            Material::Emissive {
                emit_color,
                strength,
            } => emit_color.sample(rec.uv) * strength,

            Material::Lambertian { albedo } => {
                let mut scatter_direction = rec.normal + random_in_unit_sphere().normalize();

                // Catch degenerate scatter direction
                if scatter_direction.length_squared() < 1e-8 {
                    scatter_direction = rec.normal;
                }

                let scattered_ray = Ray::new(rec.p, scatter_direction);

                attenuation * ray_color(&scattered_ray, world, depth - 1, max_bounces) / probability
            }

            Material::Metallic { albedo, fuzz } => {
                let reflected_direction = reflect(r.direction.normalize(), rec.normal);

                // Add "fuzz" by adding a small random vector
                let scattered_ray =
                    Ray::new(rec.p, reflected_direction + fuzz * random_in_unit_sphere());

                // Only scatter if the reflection is away from the surface
                if scattered_ray.direction.dot(rec.normal) > 0.0 {
                    attenuation * ray_color(&scattered_ray, world, depth - 1, max_bounces)
                        / probability
                } else {
                    Vec3A::ZERO
                }
            }

            Material::Dielectric {
                index_of_refraction,
                fuzz,
            } => {
                let refraction_ratio = if rec.front_face {
                    1.0 / index_of_refraction // Ray is entering the object
                } else {
                    *index_of_refraction // Ray is exiting the object
                };

                let unit_direction = r.direction.normalize();

                // Check for Total Internal Reflection
                let cos_theta = (-unit_direction).dot(rec.normal).min(1.0);
                let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
                let cannot_refract = refraction_ratio * sin_theta > 1.0;

                let reflectance = schlick(cos_theta, refraction_ratio);

                let scatter_direction =
                    if cannot_refract || reflectance > rand::rng().random::<f32>() {
                        // Must reflect
                        reflect(unit_direction, rec.normal)
                    } else {
                        // Can refract
                        refract(unit_direction, rec.normal, refraction_ratio)
                    };

                let scattered_ray = Ray::new(rec.p, scatter_direction);
                attenuation * ray_color(&scattered_ray, world, depth - 1, max_bounces)
            }
        };
    }

    world.background_color
}

/// Generates a random 3D vector inside a unit sphere
#[inline(always)]
fn random_in_unit_sphere() -> Vec3A {
    let mut rng = rand::rng();
    loop {
        let p = Vec3A::new(
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
        );
        if p.length_squared() < 1.0 {
            return p;
        }
    }
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
