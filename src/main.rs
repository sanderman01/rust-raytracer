extern crate image;
extern crate nalgebra_glm as glm;

use glm::{dot, normalize, vec3, Vec3};
use image::{Rgb, RgbImage};
use std::path::Path;

fn main() {
    let width = 512;
    let height = 256;
    let mut img: RgbImage = RgbImage::new(width, height);

    let camera = Camera {
        position: Vec3::new(0.0, 0.0, 1.0),
        direction: Vec3::new(0.0, 0.0, -1.0),
        field_of_view: 45.0,
    };

    let sphere = Box::new(Sphere {
        position: vec3(0.0, 0.0, -1.0),
        radius: 0.5,
    });

    let sphere1 = Box::new(Sphere {
        position: vec3(1.0, 0.0, -1.0),
        radius: 0.5,
    });

    let sphere2 = Box::new(Sphere {
        position: vec3(-1.0, 0.0, -1.0),
        radius: 0.5,
    });

    let scene: Vec<Box<SceneObject>> = vec![sphere, sphere1, sphere2];

    render_image(&camera, &scene, &mut img);

    let path = &Path::new("target/out.png");
    let _ = img.save(path);
}

fn render_image(camera: &Camera, scene: &Vec<Box<SceneObject>>, image: &mut RgbImage) {
    let width = image.width();
    let height = image.height();
    let aspect_ratio = (width as f32) / (height as f32);

    for y in 0..height {
        for x in 0..width {
            let u = x as f32 / width as f32;
            let v = y as f32 / height as f32;
            let ray = camera.screen_to_ray(u, v, aspect_ratio);
            let color = trace_ray(&ray, &scene, 0.0, std::f32::MAX);
            image.put_pixel(x, y, vec3_to_rgb(&color));
        }
    }
}

fn trace_ray(ray: &Ray, scene: &Vec<Box<SceneObject>>, t_min: f32, t_max: f32) -> Vec3 {
    // determine closest hit
    let mut closest: f32 = t_max;
    let mut result: Option<HitResult> = None;

    for obj in scene.iter() {
        match obj.ray_hit(&ray, t_min, t_max) {
            None => {}
            Some(h) => {
                if h.t < closest {
                    closest = h.t;
                    result = Some(h)
                }
            }
        }
    }

    // return color for closest hit, or background
    match result {
        Some(h) => h.normal,
        None => background_color_gradient(&ray),
    }
}

trait SceneObject {
    fn ray_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult>;
}

struct HitResult {
    t: f32,
    point: Vec3,
    normal: Vec3,
}

struct Sphere {
    position: Vec3,
    radius: f32,
}

impl SceneObject for Sphere {
    fn ray_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
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
                Some(HitResult {
                    t: t,
                    point: p,
                    normal: n,
                })
            }
        }
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
    fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

fn background_color_gradient(ray: &Ray) -> Vec3 {
    let unit_dir: Vec3 = glm::normalize(&ray.direction);
    let ground_color: Vec3 = Vec3::new(0.9, 0.9, 0.9);
    let sky_color: Vec3 = Vec3::new(0.5, 0.5, 0.5);
    let t = 1.0 * (unit_dir.y + 0.5);
    glm::lerp(&ground_color, &sky_color, t)
}

fn vec3_to_rgb(v: &Vec3) -> Rgb<u8> {
    let a = v * 255.99;
    let b = glm::clamp(&a, 0.0, 255.0);
    Rgb([b.x as u8, b.y as u8, b.z as u8])
}
