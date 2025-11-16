use crate::camera::Camera;
use crate::hittable::{Hittable, HittableObject, HitRecord};
use crate::ray::Ray;

#[derive(Debug, Clone)]
pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<HittableObject>,
}

impl Hittable for Scene {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut closest_so_far = t_max;
        let mut hit_anything = None;

        for object in &self.objects {
            if let Some(rec) = object.hit(r, t_min, closest_so_far) {
                closest_so_far = rec.t;
                hit_anything = Some(rec);
            }
        }

        hit_anything
    }
}
