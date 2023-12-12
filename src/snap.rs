use crate::error::*;
use crate::format_tran::*;

use std::env;
use std::str::FromStr;

use serde::Deserialize;

use thirtyfour::cookie::SameSite;
use thirtyfour::prelude::*;

pub struct Driver {
    pub driver: WebDriver,
}

#[derive(Deserialize, Clone)]
pub struct InForm {
    pub url: String,
    pub h: u32,
    pub w: u32,
    pub filename: String,
    pub fileformat: String,
}

pub async fn new_driver() -> SnapResult<WebDriver> {
    let ip = env::args().nth(1).unwrap_or("localhost".to_owned());
    let port = env::args().nth(2).unwrap_or("9515".to_owned());
    let ds = format!("http://{ip}:{port}");

    let mut caps = DesiredCapabilities::chrome();

    caps.set_headless()?;
    //use thirtyfour::cookie::SameSite;

    let driver = WebDriver::new(ds.as_str(), caps).await?;

    Ok(driver)
}

pub async fn take_pic(driver: WebDriver, payload: &InForm) -> SnapResult<Vec<u8>> {
    //let driver = new_driver().await;

    driver.set_window_rect(0, 0, payload.h, payload.w).await?;

    driver.goto(&payload.url).await?;
    let mut cookie = Cookie::parse_encoded("foo=bar%20baz; HttpOnly; Secure; domain=baidu.com")?;
    cookie.set_path("/");
    cookie.set_same_site(SameSite::None);
    driver.add_cookie(cookie).await?;

    let png_buffer = driver.screenshot_as_png().await?;
    let file_format = FileFormat::from_str(&payload.fileformat)?;
    let trans_buffer = png_transformer(&png_buffer, file_format)?;

    Ok(trans_buffer)
}
