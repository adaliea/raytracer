use crate::camera::Camera;
use crate::hittable::{HitRecord, Hittable, HittableObject};
use crate::ray::Ray;
use bvh::bvh::Bvh;
use glam::Vec3A;

#[derive(Debug, Clone)]
pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<HittableObject>,
    pub lights: Vec<usize>,
    pub bvh: Bvh<f32, 3>,
    pub background_color: Vec3A,
}

impl Hittable for Scene {
    #[inline(always)]
    fn hit(&'_ self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord<'_>> {
        let mut closest_so_far = t_max;
        let mut hit_anything = None;

        // let bvh_ray = bvh::ray::Ray::new(
        //     Point3::new(r.origin.x, r.origin.y, r.origin.z),
        //     Vector3::new(r.direction.x, r.direction.y, r.direction.z),
        // );
        // let hit_objs = self.bvh.traverse(&bvh_ray, &self.objects);
        for object in &self.objects {
            if let Some(rec) = object.hit(r, t_min, closest_so_far) {
                closest_so_far = rec.t;
                hit_anything = Some(rec);
            }
        }

        hit_anything
    }
}
