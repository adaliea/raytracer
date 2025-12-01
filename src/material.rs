use std::sync::Arc;
use crate::hittable::LazyUv;
use glam::Vec3A;
use image::RgbImage;

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum Material {
    /// A matte, non-reflective surface (e.g., paper, rough wood).
    /// Scatters rays in a random direction.
    Lambertian {
        /// Color of the surface
        albedo: Texture,
    },

    /// A shiny, reflective surface (e.g., metal, polished chrome).
    /// Reflects rays based on the angle of incidence.
    Metallic {
        /// Color of the reflective surface
        albedo: Texture,
        /// How "fuzzy" or rough the reflection is (0.0 = perfect mirror)
        fuzz: f32,
    },

    /// A transparent, refractive surface (e.g., glass, water).
    /// Bends rays as they pass through.
    Dielectric {
        index_of_refraction: f32, // e.g., 1.5 for glass
        /// How "fuzzy" or rough the reflection is (0.0 = perfect mirror)
        fuzz: f32,
    },

    /// A surface that emits light.
    /// This is the new variant.
    Emissive {
        /// The color of the light
        emit_color: Texture,
        /// How bright the light is
        strength: f32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Texture {
    Image(Arc<RgbImage>),
    SolidColor(Vec3A),
}

impl Texture {
    #[inline(always)]
    pub fn sample(&self, uv: &LazyUv) -> Vec3A {
        match self {
            Texture::Image(image) => {
                let uv = uv.get_uv();

                let rgb = image.get_pixel(
                    (uv.x * (image.width() - 1) as f32) as u32,
                    (uv.y * (image.height() - 1) as f32) as u32,
                );
                Vec3A::new(
                    rgb[0] as f32 / 255.0,
                    rgb[1] as f32 / 255.0,
                    rgb[2] as f32 / 255.0,
                )
            }
            Texture::SolidColor(color) => *color,
        }
    }
}
