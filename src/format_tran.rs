use strum::Display;
use strum::EnumString;

use image::load_from_memory;
use image::ImageFormat;

use std::io::Cursor;
use std::time::Instant;

#[derive(EnumString, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[strum(ascii_case_insensitive)]
pub enum FileFormat {
    Png,    // fast, small
    Jpeg,   // middle, small
    Tiff,   // fast, big,
    Webp,   // middle, small
    Tga,    // fast, big
    Bmp,    // fast, big
    Qoi,    // seems not working
    Gif,    // slow,small
    Pdf,
}

pub fn png_transformer(buf: &[u8], inform: FileFormat) -> Vec<u8> {
    let start = Instant::now();

    let png = load_from_memory(buf).unwrap();
    let mut outbuf = Vec::new();
    let mut cursor = Cursor::new(&mut outbuf);
    match inform {
        FileFormat::Png => {
            outbuf = buf.to_owned();
        }
        FileFormat::Jpeg => {
            png.write_to(&mut cursor, ImageFormat::Jpeg).unwrap();
        }
        FileFormat::Tiff => {
            png.write_to(&mut cursor, ImageFormat::Tiff).unwrap();
        }
        FileFormat::Webp => {
            png.write_to(&mut cursor, ImageFormat::WebP).unwrap();
        }
        FileFormat::Tga => {
            png.write_to(&mut cursor, ImageFormat::Tga).unwrap();
        }
        FileFormat::Qoi => {
            png.write_to(&mut cursor, ImageFormat::Qoi).unwrap();
        }
        FileFormat::Bmp => {
            png.write_to(&mut cursor, ImageFormat::Bmp).unwrap();
        }
        FileFormat::Gif => {
            png.write_to(&mut cursor, ImageFormat::Gif).unwrap();
        }

        FileFormat::Pdf=> {
            png.write_to(&mut cursor, ImageFormat::Gif).unwrap();
        }
    };
    let duration = start.elapsed();
    println!("函数运行时间: {:?}", duration);
    outbuf
}

pub fn get_content_type( inform: FileFormat ) -> String {
    match inform {
        FileFormat::Pdf => {
            format!("application/{}", inform.to_string().to_ascii_lowercase() )
        }
        _ => {
            format!("image/{}", inform.to_string().to_ascii_lowercase() )
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