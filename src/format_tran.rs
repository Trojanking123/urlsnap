use crate::error::SnapResult;
use strum::Display;
use strum::EnumString;

use image::load_from_memory;
use image::ImageFormat;

use std::io::Cursor;
use std::time::Instant;
use tracing::*;

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
        _ => {
            format!("image/{}", inform.to_string().to_ascii_lowercase())
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
