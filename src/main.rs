extern crate image;
extern crate nalgebra_glm as glm;

use glm::{dot, vec3, Vec3};
use image::{Rgb, RgbImage};
use std::path::Path;

fn main() {
    let width = 512;
    let height = 256;
    let mut img: RgbImage = RgbImage::new(width, height);

    let camera = Camera {
        position: Vec3::new(0.0, 0.0, 0.0),
        direction: Vec3::new(0.0, 0.0, -1.0),
        field_of_view: 45.0,
    };

    render_image(&camera, &mut img);

    let path = &Path::new("target/out.png");
    let _ = img.save(path);
}

fn render_image(camera: &Camera, image: &mut RgbImage) {
    let width = image.width();
    let height = image.height();
    let aspect_ratio = (width as f32) / (height as f32);

    // define rectangle through which we will shoot our rays, aka our viewport.
    let angle_rad = camera.field_of_view.to_radians();

    let rect_dist = angle_rad.cos();
    let rect_extends_y = angle_rad.sin();
    let rect_extends_x = rect_extends_y * aspect_ratio;
    let horizontal = vec3(rect_extends_x, 0.0, 0.0) * 2.0;
    let vertical = vec3(0.0, rect_extends_y, 0.0) * 2.0;

    let lower_left = vec3(-rect_extends_x, -rect_extends_y, -rect_dist);

    for y in 0..height {
        for x in 0..width {
            let u = x as f32 / width as f32;
            let v = y as f32 / height as f32;

            let origin = vec3(0.0, 0.0, 0.0);
            let direction = lower_left + horizontal * u + vertical * (1.0 - v);
            let ray = Ray { origin, direction };
            let color = trace_ray(&ray);
            image.put_pixel(x, y, vec3_to_rgb(&color));
        }
    }
}

fn trace_ray(ray: &Ray) -> Vec3 {
    let sphere_pos = vec3(0.0, 0.0, -1.0);
    let sphere_radius = 0.5;
    let t = hit_sphere(&sphere_pos, sphere_radius, &ray);
    if t > 0.0 {
        let normal = glm::normalize(&(&ray.point_at(t) - sphere_pos));
        normal
    } else {
        background_color_gradient(&ray)
    }
}

fn hit_sphere(position: &Vec3, radius: f32, ray: &Ray) -> f32 {
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
    let ac = ray.origin - position;
    let a = dot(&ray.direction, &ray.direction);
    let b = 2.0 * dot(&ac, &ray.direction);
    let c = dot(&ac, &ac) - radius * radius;
    let discr = b * b - 4.0 * a * c;
    if discr < 0.0 {
        (-1.0)
    } else {
        (-b - discr.sqrt()) / (2.0 * a)
    }
}

struct Camera {
    position: Vec3,
    direction: Vec3,
    field_of_view: f32,
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
