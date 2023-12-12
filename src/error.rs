use std::error::Error as ErrorTrait;

use thiserror::Error;

use image::error::ImageError;
use strum::ParseError;
use thirtyfour::cookie::ParseError as CookieParseError;
use thirtyfour::error::WebDriverError;

#[derive(Error, Debug)]
pub enum SnapError {
    #[error("got a driver error {0}")]
    DriverError(#[from] WebDriverError),
    #[error("got a image deal error {0}")]
    ImgError(#[from] ImageError),
    #[error("file format error {0}")]
    FormatError(#[from] ParseError),
    #[error("cookie sunpported {0}")]
    CookieParseError(#[from] CookieParseError),
    #[error("Unknown error {0}")]
    Other(#[from] Box<dyn ErrorTrait + Send + Sync>),
}

pub type SnapResult<T> = Result<T, SnapError>;
