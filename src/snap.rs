use crate::cdp::*;
use crate::error::*;
use crate::format_tran::*;

use std::env;

use std::str::FromStr;

use std::time::Duration;

use serde::Deserialize;

use thirtyfour::extensions::cdp::ChromeDevTools;
use thirtyfour::prelude::*;

use tracing::debug;
use tracing::info;

use async_trait::async_trait;

pub struct Driver {
    pub driver: WebDriver,
}

#[derive(Deserialize, Clone)]
pub struct InForm {
    pub url: String,
    pub device: Option<String>,
    pub h: u32,
    pub w: u32,
    pub filename: String,
    pub fileformat: String,
    pub cookie: Option<String>,
    pub waittime: Option<u8>,
    pub fullpage: Option<bool>,
    pub useragent: Option<String>,
    pub platform: Option<String>,
    pub lang: Option<String>,
    pub factor: Option<f32>,
    pub landscape: Option<bool>,
    pub darkmode: Option<bool>,
    pub touch: Option<bool>,
}

#[async_trait]
pub trait WebDriverOps {
    async fn new_driver() -> SnapResult<WebDriver> {
        let ip = env::args().nth(1).unwrap_or("localhost".to_owned());
        let port = env::args().nth(2).unwrap_or("9515".to_owned());
        let ds = format!("http://{ip}:{port}");

        let mut caps = DesiredCapabilities::chrome();

        caps.set_headless()?;
        caps.add_chrome_arg("--no-sandbox")?;
        
        //caps.add_chrome_arg("--remote-debugging-pipe")?;
        let aa;
        static mut  dport: u32 = 9332;
        unsafe {
            dport = dport + 1;
            dbg!(dport);
            aa = format!("--remote-debugging-port={:?}", dport);
        }
        
        caps.add_chrome_arg(aa.as_str())?;

        // info!("before setting");
        // caps.set_debugger_address("localhost:9333")?;
        // info!("after setting");

        let driver = WebDriver::new(ds.as_str(), caps).await?;
        info!("got driver");
        let dura = Some(Duration::from_secs(180));
        let tcfg = TimeoutConfiguration::new(dura, dura, dura);
        driver.update_timeouts(tcfg).await?;
        
        Ok(driver)
    }
}

impl WebDriverOps for WebDriver {}

pub async fn take_pic(driver: WebDriver, payload: &InForm) -> SnapResult<Vec<u8>> {
    //let driver = new_driver().await;
    let dev_tools = ChromeDevTools::new(driver.handle.clone());
    dev_tools.get_sinks().await?;
    let mut device = DeviceInfo::from(payload);

    if let Some(ref mut ua) = device.userAgent {
        if ua.userAgent.is_empty() {
            let browser_ua = dev_tools.get_browser_useragent().await?;
            ua.userAgent = browser_ua;
        }
        dev_tools.set_request_useragent(ua).await?;
    }

    dev_tools.set_request_device(&device).await?;
    dev_tools.set_darkmode(device.isDarkMode).await?;
    dev_tools.set_scrollbar_hidden(true).await?;

    let p = driver
        .execute("return navigator.platform", Vec::new())
        .await?;
    let p = p.convert::<String>()?;
    info!("platform: {}", p);

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
        info!(scroll_h, scroll_w);
        device.width = scroll_w;
        device.height = scroll_h;
        dev_tools.set_request_device(&device).await?;
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

    let png_buffer = driver.screenshot_as_png().await?;
    let file_format = FileFormat::from_str(&payload.fileformat)?;
    let trans_buffer = png_transformer(&png_buffer, file_format)?;

    Ok(trans_buffer)
}
