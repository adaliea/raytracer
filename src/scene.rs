use crate::camera::Camera;
use crate::hittable::{HitRecord, Hittable, HittableObject};
use crate::material::Material;
use crate::ray::Ray;

#[derive(Debug, Clone)]
pub struct Scene {
    pub camera: Camera,
    pub materials: Vec<Material>,
    pub objects: Vec<HittableObject>,
}

impl Hittable for Scene {
    fn hit(&'_ self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut closest_so_far = t_max;
        let mut hit_anything = None;

        // todo use bvh
        for object in &self.objects {
            if let Some(rec) = object.hit(r, t_min, closest_so_far) {
                closest_so_far = rec.t;
                hit_anything = Some(rec);
            }
        }

        hit_anything
    }
}
