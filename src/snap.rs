use crate::error::*;
use crate::format_tran::*;

use std::env;
use std::str::FromStr;
use std::time::Duration;

use serde::Deserialize;

use serde_json::Value;
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
    pub useragent: Option<String>,
    pub platform: Option<String>,
    pub lang: Option<String>,
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


async fn call_cdp_command(cdt: &ChromeDevTools, cmd: &str, params: Option<Value>) -> SnapResult<Option<Value>> {
    let res;
    match params {
        Some(params) => {
            res = cdt.execute_cdp_with_params(cmd, params).await;
        },
        None => {
            res = cdt.execute_cdp(cmd).await
        }
    }
    match res {
        Ok(v) => Ok(Some(v)),
        Err(e) => {
            match e {
                WebDriverError::Json(_) => Ok(None),
                _ => Err(SnapError::DriverError(e)),
                
            }
        }
    }
}


async fn get_browser_useragent(cdt: &ChromeDevTools) -> SnapResult<String> {
    let version_info = call_cdp_command(cdt, "Browser.getVersion", None).await?.unwrap();
    let user_agent = version_info["userAgent"].as_str().unwrap().to_owned();
    info!("user agent: {}", user_agent);
    Ok(user_agent)
}



async fn set_request_useragent(cdt: &ChromeDevTools, payload: &InForm) -> SnapResult<()> {
    
    let user_agent = match payload.useragent {
        Some(_) => payload.useragent.clone().unwrap(),
        None => get_browser_useragent(cdt).await?
    };
    let lang = payload.lang.clone().unwrap_or("".to_owned());
    let platform = payload.platform.clone().unwrap_or("".to_owned());

    dbg!("{}", &user_agent);

    let args = json!({"userAgent": user_agent ,"acceptLanguage": lang,"platform": platform});
    call_cdp_command(cdt, "Emulation.setUserAgentOverride", Some(args)).await?;

    let args = json!({"enabled": true});
    call_cdp_command(cdt, "Emulation.setAutoDarkModeOverride", Some(args)).await?;

    let args = json!({"enabled": false});
    call_cdp_command(cdt, "Emulation.setAutomationOverride", Some(args)).await?;
    
    let args = json!({"width": 428, "height": 926, "deviceScaleFactor": 3, "mobile": true, });
    call_cdp_command(cdt, "Emulation.setDeviceMetricsOverride", Some(args)).await?;

    let args = json!({"enabled": false});
    call_cdp_command(cdt, "Emulation.setTouchEmulationEnabled", Some(args)).await?;

    Ok(())
}

pub async fn take_pic(driver: WebDriver, payload: &InForm) -> SnapResult<Vec<u8>> {
    //let driver = new_driver().await;
    let dev_tools = ChromeDevTools::new(driver.handle.clone());

    set_request_useragent(&dev_tools, &payload).await?;

    let p = driver.execute("return navigator.platform", Vec::new()).await?;
    let p = p.convert::<String>()?;
    info!("platform: {}", p);

    //driver.set_window_rect(0, 0, payload.h, payload.w).await?;

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


// Emulation.setAutoDarkModeOverride 
// Emulation.setAutomationOverride