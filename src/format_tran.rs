use crate::error::SnapResult;
use image::DynamicImage;
use image::ImageBuffer;
use image::Rgba;
use strum::Display;
use strum::EnumString;

use image::load_from_memory;
use image::ImageFormat;
use vtracer::convert;
use vtracer::ColorImage;

use std::io::Cursor;
use std::time::Instant;
use tracing::*;


fn dynamic_image_to_image_buffer(dynamic_img: DynamicImage) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    // 获取 DynamicImage 的像素数据
    let width = dynamic_img.width();
    let height = dynamic_img.height();
    let rgba_image = dynamic_img.to_rgba8();

    // 创建一个新的 ImageBuffer，并将像素数据复制到其中
    let mut img_buffer = ImageBuffer::new(width, height);
    for (x, y, pixel) in img_buffer.enumerate_pixels_mut() {
        *pixel = rgba_image.get_pixel(x, y).to_owned();
    }

    // 返回转换后的 ImageBuffer
    img_buffer
}

#[derive(EnumString, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[strum(ascii_case_insensitive)]
pub enum FileFormat {
    Png,  // fast, small
    Jpeg, // middle, small
    Tiff, // fast, big,
    Webp, // middle, small
    Tga,  // fast, big
    Bmp,  // fast, big
    Qoi,  // seems not working
    Gif,  // slow,small
    Svg,
    Pdf,
}

pub fn png_transformer(buf: &[u8], inform: FileFormat) -> SnapResult<Vec<u8>> {
    let start = Instant::now();

    let png = load_from_memory(buf)?;
    let mut outbuf = Vec::new();
    let mut cursor = Cursor::new(&mut outbuf);
    match inform {
        FileFormat::Png => {
            outbuf = buf.to_owned();
        }
        FileFormat::Jpeg => {
            png.write_to(&mut cursor, ImageFormat::Jpeg)?;
        }
        FileFormat::Tiff => {
            png.write_to(&mut cursor, ImageFormat::Tiff)?;
        }
        FileFormat::Webp => {
            png.write_to(&mut cursor, ImageFormat::WebP)?;
        }
        FileFormat::Tga => {
            png.write_to(&mut cursor, ImageFormat::Tga)?;
        }
        FileFormat::Qoi => {
            png.write_to(&mut cursor, ImageFormat::Qoi)?;
        }
        FileFormat::Bmp => {
            png.write_to(&mut cursor, ImageFormat::Bmp)?;
        }
        FileFormat::Gif => {
            png.write_to(&mut cursor, ImageFormat::Gif)?;
        }

        FileFormat::Pdf => {
            png.write_to(&mut cursor, ImageFormat::Gif)?;
        }

        FileFormat::Svg => {
            let h = png.height();
            let w = png.width();
            let mut img = ColorImage::new_w_h(w as usize, h as usize);
            let img_buffer = dynamic_image_to_image_buffer(png);
            img.pixels = img_buffer.as_raw().to_owned();
            let cfg = vtracer::Config::default();
            let svg = convert(img, cfg).unwrap();
            let svg_str = svg.to_string();
            outbuf.extend(svg_str.as_bytes());
        }
    };
    let duration = start.elapsed();
    info!("picture size {}", outbuf.len());
    info!("picture trans cost time {:?}", duration);
    Ok(outbuf)
}

pub fn get_content_type(inform: FileFormat) -> String {
    match inform {
        FileFormat::Pdf => {
            format!("application/{}", inform.to_string().to_ascii_lowercase())
        }
        FileFormat::Svg => {
            format!("image/svg+xml")
        }
        _ => {
            format!("image/pdf")
        }
    }
}

#[allow(unused_imports)]
mod test {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn testtt() {
        assert_eq!(FileFormat::Png, FileFormat::from_str("Png").unwrap());
        assert_eq!(FileFormat::Png, FileFormat::from_str("png").unwrap());
        assert_eq!(FileFormat::Png.to_string().to_ascii_lowercase(), "Png");
    }
}
