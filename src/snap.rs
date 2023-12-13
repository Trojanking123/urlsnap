use crate::error::*;
use crate::format_tran::*;

use std::env;
use std::str::FromStr;
use std::time::Duration;

use serde::Deserialize;

use serde_json::json;

use thirtyfour::extensions::cdp::ChromeDevTools;
use thirtyfour::prelude::*;

use tracing::info;

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
    pub cookie: Option<String>,
    pub waittime: Option<u8>,
    pub fullpage: Option<bool>,
}

pub async fn new_driver() -> SnapResult<WebDriver> {
    let ip = env::args().nth(1).unwrap_or("localhost".to_owned());
    let port = env::args().nth(2).unwrap_or("9515".to_owned());
    let ds = format!("http://{ip}:{port}");

    let mut caps = DesiredCapabilities::chrome();

    caps.set_headless()?;

    let driver = WebDriver::new(ds.as_str(), caps).await?;

    Ok(driver)
}

pub async fn take_pic(driver: WebDriver, payload: &InForm) -> SnapResult<Vec<u8>> {
    //let driver = new_driver().await;
    let dev_tools = ChromeDevTools::new(driver.handle.clone());
    // let version_info = dev_tools.execute_cdp("Browser.getVersion").await?;
    // let user_agent = version_info["userAgent"].as_str().unwrap();
    // info!("user agent: {}", user_agent);

    let args = json!({"userAgent": "Mozilla/5.0 (iPhone; CPU iPhone OS 8_4 like Mac OS X) AppleWebKit/600.1.4 (KHTML, like Gecko) Mobile/12H143","acceptLanguage": "en-US","platform": "iPhone"});
    let b = serde_json::to_string(&args).unwrap();
    dbg!(b);
    dbg!(args.clone());
    // dev_tools
    //     .execute_cdp_with_params("Network.setUserAgentOverride", args)
    //     .await
    //     .unwrap();

    dev_tools.execute_cdp_with_params("Network.setCacheDisabled", json!({"cacheDisabled": true})).await?;
    let p = driver.execute("return navigator.lang", Vec::new()).await?;
    let p = p.convert::<String>()?;
    info!("platform: {}", p);

    driver.set_window_rect(0, 0, payload.h, payload.w).await?;

    driver.goto(&payload.url).await?;
    if let Some(true) = payload.fullpage {
        let scroll_w = driver
            .execute("return document.body.parentNode.scrollWidth", Vec::new())
            .await?
            .convert::<u32>()?;
        let scroll_h = driver
            .execute("return document.body.parentNode.scrollHeight", Vec::new())
            .await?
            .convert::<u32>()?;
        info!("{} {}", scroll_h, scroll_w);
        driver.set_window_rect(0, 0, scroll_w, scroll_h).await?;
    }
    if let Some(ref cookie) = payload.cookie {
        let cookie = Cookie::parse_encoded(cookie)?.into_owned();
        driver.add_cookie(cookie).await?;
        driver.refresh().await?;
    }

    if let Some(waittime) = payload.waittime {
        let delay = Duration::new(waittime as u64, 0);
        tokio::time::sleep(delay).await;
    }

    let elem = driver.find(By::Tag("body")).await?;
    let png_buffer = elem.screenshot_as_png().await?;
    let file_format = FileFormat::from_str(&payload.fileformat)?;
    let trans_buffer = png_transformer(&png_buffer, file_format)?;

    Ok(trans_buffer)
}
