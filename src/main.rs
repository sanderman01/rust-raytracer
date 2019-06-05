extern crate image;
extern crate nalgebra_glm as glm;
extern crate rand;
extern crate rayon;

use glm::{dot, normalize, vec3, Vec3};
use image::{Rgb, RgbImage};
use rand::Rng;
use rayon::prelude::*;
use std::path::Path;
use std::time::Instant;

fn main() {
    let width = 2048;
    let height = 1024;
    let mut img: RgbImage = RgbImage::new(width, height);

    let camera = Camera {
        position: Vec3::new(0.0, 0.0, 1.0),
        direction: Vec3::new(0.0, 0.0, -1.0),
        field_of_view: 45.0,
    };

    let sphere = Box::new(Sphere {
        position: vec3(0.0, 0.0, -1.0),
        radius: 1.0,
        material: Box::new(Metal {
            albedo: vec3(0.5, 1.0, 0.5),
            scattering: 0.2,
        }),
    });
    let sphere1 = Box::new(Sphere {
        position: vec3(2.0, 0.0, -1.0),
        radius: 1.0,
        material: Box::new(Metal {
            albedo: vec3(0.5, 0.5, 1.0),
            scattering: 0.0,
        }),
    });
    let sphere2 = Box::new(Sphere {
        position: vec3(-2.0, 0.0, -1.0),
        radius: 1.0,
        material: Box::new(Diffuse {
            albedo: vec3(1.0, 0.5, 0.5),
        }),
    });
    let sphere3 = Box::new(Sphere {
        position: vec3(0.0, -101.0, 0.0),
        radius: 100.0,
        material: Box::new(Diffuse {
            albedo: vec3(1.0, 1.0, 1.0),
        }),
    });

    let scene: Vec<Box<dyn SceneObject>> = vec![sphere, sphere1, sphere2, sphere3];

    let now = Instant::now();
    render_image(&camera, &scene, &mut img);
    let duration = now.elapsed().as_secs();
    println!("rendering image took {:.2}s", duration);

    let path = &Path::new("target/out.png");
    let _ = img.save(path);
}

fn render_image(camera: &Camera, scene: &Vec<Box<SceneObject>>, image: &mut RgbImage) {
    let width = image.width();
    let height = image.height();
    let aspect_ratio = (width as f32) / (height as f32);
    let num_samples = 64;
    let pixel_indices: Vec<(u32, u32, &mut Rgb<u8>)> = image.enumerate_pixels_mut().collect();

    // for each pixel, shoot rays to determine color.
    // use rayon's parallel iterator to divide work over all cpu cores.
    let pixels: Vec<(u32, u32, Rgb<u8>)> = pixel_indices
        .par_iter()
        .map(|(x, y, _)| {
            // do a number of ray samples
            let mut rng = rand::thread_rng();
            let mut total_color = vec3(0.0, 0.0, 0.0);
            for _s in 0..num_samples {
                let u: f32 = (*x as f32 + rng.gen::<f32>()) / width as f32;
                let v: f32 = (*y as f32 + rng.gen::<f32>()) / height as f32;
                let ray = camera.screen_to_ray(u, v, aspect_ratio);
                let depth = 0;
                let color = trace_ray(&ray, &scene, depth);
                total_color += color;
            }
            // determine final result pixel color
            let total_color = total_color / num_samples as f32;
            let final_color = encode_gamma(&total_color, 2.2);
            let c: Rgb<u8> = vec3_to_rgb(&final_color);
            (*x, *y, c)
        })
        .collect();

    // write pixel data to image
    for (x, y, c) in pixels {
        image.put_pixel(x, y, c);
    }
}

fn scene_hit<'a>(
    ray: &Ray,
    scene: &'a Vec<Box<SceneObject>>,
    min_t: f32,
    max_t: f32,
) -> Option<HitRecord<'a>> {
    // determine closest hit
    let mut closest: f32 = std::f32::MAX;
    let mut result: Option<HitRecord> = None;
    for obj in scene.iter() {
        match obj.ray_hit(&ray, min_t, max_t) {
            None => {}
            Some(h) => {
                if h.t < closest {
                    closest = h.t;
                    result = Some(h)
                }
            }
        }
    }
    result
}

fn trace_ray(ray: &Ray, scene: &Vec<Box<SceneObject>>, depth: u32) -> Vec3 {
    let hit = scene_hit(ray, scene, 0.001, std::f32::MAX);
    // return color for closest hit, or background
    match hit {
        // we hit something, so do a bounce in a random direction
        Some(h) => {
            if depth < 64 {
                let (atten, scattered_ray) = h.material.scatter(&ray, &h);
                match scattered_ray {
                    Some(r) => {
                        let col = trace_ray(&r, &scene, depth + 1);
                        vec3(atten.x * col.x, atten.y * col.y, atten.z * col.z)
                    }
                    None => vec3(0.0, 0.0, 0.0),
                }
            } else {
                vec3(0.0, 0.0, 0.0)
            }
        }
        // we did not hit anything, so return background color
        None => background_color_gradient(&ray),
    }
}

trait SceneObject: Sync + Send {
    fn ray_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
    fn get_material<'a>(&'a self) -> &'a Box<dyn Material>;
}

struct Sphere {
    position: Vec3,
    radius: f32,
    material: Box<dyn Material>,
}

impl SceneObject for Sphere {
    fn ray_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        // define sphere with center C and radius R where all point P on sphere satisfy:
        //            ||P-C||^2 = R^2
        // or: dot((P-C),(P-C)) = R^2
        //
        // determine intersection with Ray by replacing P with A+tB and solving for t:
        // dot((A+tB-C), (A+tB-C)) = R^2
        // t^2 * dot(B,B) + 2t * dot(B,A-C) + dot(A-C,A-C) - r^2 = 0
        //
        // which we can also write as:
        // a(t^2) + bt + c = 0
        // where
        // a = dot(B,B)
        // b = 2*dot(B,A-C)
        // c = dot(A-C,A-C) - r^2
        //
        // so we can use the standard abc quadratic formula
        //     -b +- srt(b^2 - 4ac)
        // t = --------------------
        //             2a
        let ac = ray.origin - self.position;
        let a = dot(&ray.direction, &ray.direction);
        let b = 2.0 * dot(&ac, &ray.direction);
        let c = dot(&ac, &ac) - self.radius * self.radius;
        let discr = b * b - 4.0 * a * c;
        if discr < 0.0 {
            None
        } else {
            let t = (-b - discr.sqrt()) / (2.0 * a);
            if t > t_max || t < t_min {
                None
            } else {
                let p = ray.point_at(t);
                let n = normalize(&(p - self.position));
                Some(HitRecord {
                    t: t,
                    point: p,
                    normal: n,
                    material: &self.material,
                })
            }
        }
    }

    fn get_material<'a>(&'a self) -> &'a Box<Material> {
        &self.material
    }
}

struct Camera {
    position: Vec3,
    direction: Vec3,
    field_of_view: f32,
}

impl Camera {
    fn screen_to_ray(&self, u: f32, v: f32, aspect_ratio: f32) -> Ray {
        // define rectangle through which we will shoot our rays, aka our viewport.
        let angle_rad = self.field_of_view.to_radians();

        let rect_dist = angle_rad.cos();
        let rect_extends_y = angle_rad.sin();
        let rect_extends_x = rect_extends_y * aspect_ratio;
        let horizontal = vec3(rect_extends_x, 0.0, 0.0) * 2.0;
        let vertical = vec3(0.0, rect_extends_y, 0.0) * 2.0;

        let lower_left = vec3(-rect_extends_x, -rect_extends_y, -rect_dist);

        let origin = self.position;
        let direction = lower_left + horizontal * u + vertical * (1.0 - v);

        Ray { origin, direction }
    }
}

struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    fn new(origin: Vec3, direction: Vec3) -> Ray {
        Ray { origin, direction }
    }

    fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

fn background_color_gradient(ray: &Ray) -> Vec3 {
    let unit_dir: Vec3 = glm::normalize(&ray.direction);
    let ground_color: Vec3 = Vec3::new(1.0, 1.0, 1.0);
    let sky_color: Vec3 = Vec3::new(0.1, 0.2, 1.0);
    let t = 0.5 * (unit_dir.y + 1.0);
    glm::lerp(&ground_color, &sky_color, t)
}

fn vec3_to_rgb(v: &Vec3) -> Rgb<u8> {
    let a = v * 255.99;
    let b = glm::clamp(&a, 0.0, 255.0);
    Rgb([b.x as u8, b.y as u8, b.z as u8])
}

fn rand_unit_sphere() -> Vec3 {
    let mut rng = rand::thread_rng();
    let mut p: Vec3 = vec3(1.0, 1.0, 1.0);
    while glm::length2(&p) > 1.0 {
        p = 2.0 * Vec3::new(rng.gen(), rng.gen(), rng.gen()) - vec3(1.0, 1.0, 1.0);
    }
    p
}

fn encode_gamma(color: &Vec3, gamma: f32) -> Vec3 {
    let inv = 1.0 / gamma;
    let exp = vec3(inv, inv, inv);
    glm::pow(color, &exp)
}

fn reflect(v: &Vec3, n: &Vec3) -> Vec3 {
    v - 2.0 * dot(v, n) * n
}

struct HitRecord<'a> {
    t: f32,
    point: Vec3,
    normal: Vec3,
    material: &'a Box<Material>,
}

trait Material: Send + Sync {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> (Vec3, Option<Ray>);
}

struct Diffuse {
    albedo: Vec3,
}

impl Material for Diffuse {
    fn scatter(&self, _ray: &Ray, hit: &HitRecord) -> (Vec3, Option<Ray>) {
        let attenuation = self.albedo;
        let ray = Ray {
            origin: hit.point,
            direction: hit.normal + rand_unit_sphere(),
        };
        (attenuation, Some(ray))
    }
}

impl Default for Diffuse {
    fn default() -> Self {
        Diffuse {
            albedo: vec3(1.0, 1.0, 1.0),
        }
    }
}

struct Metal {
    albedo: Vec3,
    scattering: f32,
}

impl Material for Metal {
    fn scatter(&self, _ray: &Ray, hit: &HitRecord) -> (Vec3, Option<Ray>) {
        let reflected = reflect(&_ray.direction, &hit.normal);
        let scattered_ray = Ray::new(hit.point, reflected + self.scattering * rand_unit_sphere());
        let attenuation = self.albedo;

        if dot(&scattered_ray.direction, &hit.normal) > 0.0 {
            (attenuation, Some(scattered_ray))
        } else {
            (attenuation, None)
        }
    }
}
