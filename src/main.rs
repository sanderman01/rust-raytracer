extern crate image;
extern crate nalgebra_glm as glm;

use glm::{vec3, Vec3};
use image::{Rgb, RgbImage};
use std::path::Path;

fn main() {
    let width = 256;
    let height = 128;
    let mut img: RgbImage = RgbImage::new(width, height);

    let camera = Camera {
        position: Vec3::new(0.0, 0.0, 0.0),
        direction: Vec3::new(0.0, 0.0, -1.0),
        field_of_view: 30.0,
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

            let color = background_color_gradient(&ray);
            let color = vec3_to_rgb(&color);
            image.put_pixel(x, y, color);
        }
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
    let vv = v * 255.99;
    Rgb([vv.x as u8, vv.y as u8, vv.z as u8])
}
