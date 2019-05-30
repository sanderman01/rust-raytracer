extern crate image;

use std::path::Path;
use image::{RgbImage, Rgb};

fn main() {

    let width = 1024;
    let height = 1024;
    let mut buf: Vec<u8> = Vec::new();
    let mut img: RgbImage = RgbImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let u = (x as f32) / (width as f32);
            let v = (y as f32) / (height as f32);

            let r = (u * 255.99) as u8;
            let g = (v * 255.99) as u8;
            let b = 0 as u8;

            let p = Rgb([r, g, b]);
            img.put_pixel(x, y, p);
        }
    }

    let path = &Path::new("target/out.png");
    img.save(path);
}
