use crate::hittable::LazyUv;
use glam::Vec3A;
use image::Rgb32FImage;
use std::sync::Arc;

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum Material {
    /// A matte, non-reflective surface (e.g., paper, rough wood).
    /// Scatters rays in a random direction.
    Lambertian {
        /// Color of the surface
        albedo: Texture,
        normal_map: Option<Texture>,
    },

    /// A shiny, reflective surface (e.g., metal, polished chrome).
    /// Reflects rays based on the angle of incidence.
    Metallic {
        /// Color of the reflective surface
        albedo: Texture,
        normal_map: Option<Texture>,
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
    Image(Arc<Rgb32FImage>),
    SolidColor(Vec3A),
}

impl Texture {
    #[inline(always)]
    pub fn sample(&self, uv: &LazyUv) -> Vec3A {
        match self {
            Texture::Image(image) => {
                let uv = uv.get_uv();

                // Use clamp to prevent out of bounds access
                let u = uv.x.rem_euclid(1.0);
                // Flip v for image coords
                let v = 1.0 - uv.y.rem_euclid(1.0);

                let x = (u * (image.width() - 1) as f32) as u32;
                let y = (v * (image.height() - 1) as f32) as u32;

                let pixel = image.get_pixel(x, y);
                Vec3A::new(pixel[0], pixel[1], pixel[2])
            }
            Texture::SolidColor(color) => *color,
        }
    }
}
